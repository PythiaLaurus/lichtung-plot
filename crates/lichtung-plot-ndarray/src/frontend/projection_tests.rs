use ndarray::{array, s, Array2, ArrayView1};

use crate::{
    AdapterError, ArraySource, Boxplot, CategoryValues, ColumnProjection, Heatmap, Labels, Line,
    Matrix, NumericValues, Plot, Scatter, SourceRowsInput,
};

fn no_other_keys() -> std::iter::Empty<(&'static str, ArrayView1<'static, &'static str>)> {
    std::iter::empty()
}

fn selected_line() -> Line<'static> {
    let x = array![[0.], [5.], [20.]];
    let seven = array![[4.], [5.], [6.]];
    let two = array![[1.], [2.], [3.]];
    let x_view = x.view();
    let seven_view = seven.view();
    let two_view = two.view();
    let entries: [(usize, &dyn ArraySource); 3] = [(4, &x_view), (7, &seven_view), (2, &two_view)];
    let identities = [(4, "Variable"), (7, "seven"), (2, "two")];
    let source = ColumnProjection::new((3, 10), &entries)
        .unwrap()
        .with_identities(&identities)
        .unwrap();
    let a = Line::from_data(source, 4usize, [7usize, 2usize])
        .unwrap()
        .labels(Labels::raw().alias("Variable", "Time"))
        .column_labels(["Right", "Left"])
        .unwrap();
    let (plot, metadata) = a.into_parts();
    Line::from_parts(plot, metadata)
}

#[test]
fn reordered_identity_context_binds_only_fresh_roles_without_renaming_old_values() {
    let (plot, metadata) = selected_line().into_parts();
    // The old numeric key "two" now occupies column 1 and contains categories.
    // None of the original x/y views or their value entries survive this call.
    let categories = array![["A"], ["B"], ["A"]];
    let categories_view = categories.view();
    let entries: [(usize, &dyn ArraySource); 1] = [(1, &categories_view)];
    let identities = [(1, "two")];
    let source = ColumnProjection::new((3, 11), &entries)
        .unwrap()
        .with_identities(&identities)
        .unwrap();
    let a = Line::from_parts_with_data(plot, metadata, source).unwrap();
    let repeated = a.category_rows(1usize, "hue").unwrap();
    assert_eq!(repeated.name(), "two");
    assert_eq!(repeated.len(), 6);
    let a = a.hue(1usize).unwrap();
    let (plot, metadata) = a.into_parts();

    // A genuinely new key then occupies the old x index 4.
    let categories = array![["B"], ["A"], ["B"]];
    let categories_view = categories.view();
    let entries: [(usize, &dyn ArraySource); 1] = [(4, &categories_view)];
    let identities = [(4, "new")];
    let source = ColumnProjection::new((3, 11), &entries)
        .unwrap()
        .with_identities(&identities)
        .unwrap();
    let a = Line::from_parts_with_data(plot, metadata, source).unwrap();
    let labels = a.bindings.labels.clone().alias("new", "New");
    let a = a.hue(4usize).unwrap().labels(labels);
    assert_eq!(a.bindings.labels.resolve("Variable").unwrap(), "Time");
    assert_eq!(
        a.bindings.expansion.as_ref().unwrap().keys,
        ["seven", "two"]
    );
    let expected = Plot::line(
        NumericValues::new("Variable", [0., 0., 5., 5., 20., 20.]),
        NumericValues::new("y", [4., 1., 5., 2., 6., 3.]),
    )
    .group(
        CategoryValues::new(
            "Variable_",
            ["seven", "two", "seven", "two", "seven", "two"],
        )
        .unwrap(),
    )
    .hue(CategoryValues::new("new", ["B", "B", "A", "A", "B", "B"]).unwrap())
    .labels(Labels::raw().aliases([
        ("Variable", "Time"),
        ("seven", "Right"),
        ("two", "Left"),
        ("new", "New"),
    ]));
    assert_eq!(
        crate::tests::render(&a.into(), "projection-fresh"),
        crate::tests::render(&expected, "projection-fresh-native")
    );
}

#[test]
fn every_wide_origin_avoids_unselected_identity_only_fields_and_keeps_one_column_collections() {
    let numeric = array![[0., 4.], [5., 5.], [20., 6.]];
    let categories = array![["a"], ["b"], ["a"]];
    let x = numeric.slice(s![.., ..1]);
    let y = numeric.slice(s![.., 1..2]);
    let category = categories.view();
    let entries: [(usize, &dyn ArraySource); 3] = [(4, &x), (7, &y), (2, &category)];
    let identities = [
        (4, "time"),
        (7, "seven"),
        (2, "category"),
        (0, "Variable"),
        (1, "Variable_"),
        (3, "Variable__"),
    ];
    let source = || {
        ColumnProjection::new((3, 10), &entries)
            .unwrap()
            .with_identities(&identities)
            .unwrap()
    };
    let actual_x = Line::from_data(source(), 4usize, [7usize]).unwrap();
    assert_eq!(
        actual_x.bindings.expansion.as_ref().unwrap().field,
        "Variable___"
    );
    assert_eq!(
        actual_x.bindings.expansion.as_ref().unwrap().keys,
        ["seven"]
    );
    let faceted = actual_x.facet_columns_row().unwrap();
    assert!(faceted.plot.preflight().is_ok());
    assert!(Line::from_data(source(), 4usize, 7usize)
        .unwrap()
        .bindings
        .expansion
        .is_none());

    let indexed = Scatter::indexed_from_data(source(), [7]).unwrap();
    assert_eq!(
        indexed.bindings.expansion.as_ref().unwrap().field,
        "Variable___"
    );
    assert_eq!(indexed.bindings.expansion.as_ref().unwrap().keys, ["seven"]);
    let samples = Boxplot::from_data(source(), 2usize, [7usize]).unwrap();
    assert_eq!(
        samples.bindings.expansion.as_ref().unwrap().field,
        "Variable___"
    );
    assert_eq!(samples.bindings.expansion.as_ref().unwrap().keys, ["seven"]);
    let columns = Boxplot::columns_from_data(source(), [7]).unwrap();
    assert_eq!(
        columns.bindings.expansion.as_ref().unwrap().field,
        "Variable___"
    );
    assert_eq!(columns.bindings.expansion.as_ref().unwrap().keys, ["seven"]);
}

#[test]
fn generated_alpha_resolves_identity_only_targets_without_reading_their_values() {
    let (plot, metadata) = selected_line().into_parts();
    let keys = array![[7usize], [2], [5]];
    let keys_view = keys.view();
    let entries: [(usize, &dyn ArraySource); 1] = [(9, &keys_view)];
    let identities = [(9, "keys"), (7, "seven"), (2, "two"), (5, "unused")];
    let source = || {
        ColumnProjection::new((3, 10), &entries)
            .unwrap()
            .with_identities(&identities)
            .unwrap()
    };
    let a = Line::from_parts_with_data(plot, metadata, source()).unwrap();
    let bytes = array![80u8, 220, 0];
    let main = a
        .clone()
        .line_alpha_columns(9usize, no_other_keys(), bytes.view())
        .unwrap();
    assert!(main.plot.preflight().is_ok());
    // The pooled target 5 has an identity but no values and no actual series.
    // The independent route must use the same saved generated field and mapping.
    let independent = a
        .line_alpha_columns_from_data(source(), 9usize, no_other_keys(), bytes.view())
        .unwrap();
    assert!(independent.plot.preflight().is_ok());
}

#[test]
fn generated_alpha_keeps_missing_transport_and_native_tuple_validation_distinct() {
    let (plot, metadata) = selected_line().into_parts();
    let keys = array![[7usize], [2]];
    let keys_view = keys.view();
    let entries: [(usize, &dyn ArraySource); 1] = [(9, &keys_view)];
    let identities = [(9, "keys"), (7, "seven"), (2, "two")];
    let source = ColumnProjection::new((2, 10), &entries)
        .unwrap()
        .with_identities(&identities)
        .unwrap();
    let a = Line::from_parts_with_data(plot, metadata, source).unwrap();
    let missing = [(9, "keys"), (7, "seven")];
    let independent = ColumnProjection::new((2, 10), &entries)
        .unwrap()
        .with_identities(&missing)
        .unwrap();
    let bytes = array![80u8, 220];
    assert!(matches!(
        a.clone().line_alpha_columns_from_data(independent, 9usize, no_other_keys(), bytes.view()),
        Err(AdapterError::Binding { role: "column alpha selector", detail })
            if detail.contains("column 2")
    )); // no fallback to the complete main-source table
    assert!(matches!(
        a.clone()
            .line_alpha_columns(array![10usize].view(), no_other_keys(), 80u8),
        Err(AdapterError::Column {
            index: 10,
            columns: 10,
            ..
        })
    ));
    assert!(matches!(
        a.clone().line_alpha_columns(array![7usize, 7].view(), no_other_keys(), bytes.view()),
        Err(AdapterError::Native(crate::Error::Data(detail)))
            if detail.contains("duplicate selector tuples")
    )); // raw column targets are not globally unique SourceRows
    let incomplete = a
        .line_alpha_columns(array![7usize].view(), no_other_keys(), 80u8)
        .unwrap();
    assert!(matches!(
        incomplete.plot.preflight(),
        Err(crate::Error::Data(detail)) if detail.contains("no value for actual raw unit")
    ));
}

#[test]
fn identity_context_does_not_replace_direct_local_matrix_or_column_alpha_ordinals() {
    let x = array![[0.], [5.], [20.]];
    let x_view = x.view();
    let direct = array![[1., 4.], [2., 5.], [3., 6.]];
    let entries: [(usize, &dyn ArraySource); 1] = [(4, &x_view)];
    let identities = [(4, "time"), (0, "not-local-zero"), (1, "not-local-one")];
    let source = || {
        ColumnProjection::new((3, 10), &entries)
            .unwrap()
            .with_identities(&identities)
            .unwrap()
    };
    let local = Line::from_data(source(), 4usize, direct.view()).unwrap();
    let ordinary = Line::new(x.column(0), direct.view()).unwrap();
    assert_eq!(
        local.bindings.expansion.as_ref().unwrap().keys,
        ordinary.bindings.expansion.as_ref().unwrap().keys
    );
    assert_eq!(local.bindings.labels, ordinary.bindings.labels);
    let a = local
        .line_alpha_columns(
            array![0usize, 1].view(),
            no_other_keys(),
            array![80u8, 220].view(),
        )
        .unwrap();
    assert!(a.plot.preflight().is_ok());
    let matrix = Matrix::from_data(source(), direct.view()).unwrap();
    let ordinary = Matrix::new(direct.view()).unwrap();
    assert_eq!(matrix.bindings.matrix_keys, ordinary.bindings.matrix_keys);
    assert_eq!(matrix.bindings.labels, ordinary.bindings.labels);
    assert!(matrix.plot.preflight().is_ok());
}

#[test]
fn matrix_raw_names_and_column_labels_share_transport_without_early_name_validation() {
    let values = array![[1., 4.], [2., 5.], [3., 6.]];
    let seven = values.slice(s![.., 1..2]);
    let two = values.slice(s![.., ..1]);
    let entries: [(usize, &dyn ArraySource); 2] = [(2, &two), (7, &seven)];
    let identities = [(2, "two"), (7, "seven")];
    let source = ColumnProjection::new((3, 10), &entries)
        .unwrap()
        .with_identities(&identities)
        .unwrap();
    let a = Matrix::from_data(source, [7, 2])
        .unwrap()
        .labels(Labels::raw())
        .column_labels(["Seven", "Two"])
        .unwrap();
    assert_eq!(a.bindings.matrix_keys.as_deref().unwrap(), ["seven", "two"]);
    assert_eq!(
        a.bindings.labels,
        Labels::raw().aliases([("seven", "Seven"), ("two", "Two")])
    );
    assert!(a.plot.preflight().is_ok());

    let invalid_names = [(7, ""), (2, " ")];
    let source = ColumnProjection::new((3, 10), &entries)
        .unwrap()
        .with_identities(&invalid_names)
        .unwrap();
    let a = Matrix::from_data(source, [7, 2])
        .unwrap()
        .column_labels(["Visible seven", "Visible two"])
        .unwrap();
    assert!(matches!(
        a.plot.preflight(),
        Err(crate::Error::Data(detail)) if detail.contains("variable names must be nonempty")
    ));
}

#[test]
fn grid_positions_and_per_cell_ids_keep_original_origins_and_two_dimensional_shape() {
    let values = array![[1, 2, 3], [4, 5, 6]];
    let ids = array![[101usize, 102, 103], [104, 105, 106]];
    let seven = values.slice(s![.., ..1]);
    let two = values.slice(s![.., 1..2]);
    let four = values.slice(s![.., 2..3]);
    let id9 = ids.slice(s![.., ..1]);
    let id8 = ids.slice(s![.., 1..2]);
    let id6 = ids.slice(s![.., 2..3]);
    let entries: [(usize, &dyn ArraySource); 6] = [
        (4, &four),
        (2, &two),
        (7, &seven),
        (9, &id9),
        (8, &id8),
        (6, &id6),
    ];
    let identities = [
        (4, "c"),
        (2, "b"),
        (7, "a"),
        (9, "id-a"),
        (8, "id-b"),
        (6, "id-c"),
    ];
    let source = ColumnProjection::new((2, 10), &entries)
        .unwrap()
        .with_identities(&identities)
        .unwrap();
    let a = Heatmap::from_data(source, [7, 2, 4]).unwrap();
    assert_eq!(a.bindings.grid.as_ref().unwrap().columns(), 3);
    assert_eq!(
        a.bindings.labels,
        Labels::default().aliases([
            ("__lichtung_grid_x_0", "Column 8"),
            ("__lichtung_grid_x_1", "Column 3"),
            ("__lichtung_grid_x_2", "Column 5"),
            ("__lichtung_grid_y_0", "Row 1"),
            ("__lichtung_grid_y_1", "Row 2"),
        ])
    );
    assert_eq!(
        [9, 8, 6]
            .raw_source_rows(a.source.as_deref(), "cell IDs")
            .unwrap(),
        [101, 102, 103, 104, 105, 106]
    );
    let wrong = Array2::from_elem((3, 2), false);
    assert!(matches!(
        a.clone().heatmap_highlight(wrong.view()),
        Err(AdapterError::Binding { detail, .. })
            if detail.contains("[7, 2, 4]") && detail.contains("(3, 2)")
    ));
    let wrong_ids = Array2::<usize>::zeros((3, 2));
    assert!(matches!(
        a.clone().source_rows(wrong_ids.view()),
        Err(AdapterError::Binding { detail, .. }) if detail.contains("(3, 2)")
    ));
    assert!(matches!(
        a.clone().source_rows(9usize),
        Err(AdapterError::Binding { detail, .. }) if detail.contains("not repeated row IDs")
    ));
    let a = a.source_rows([9, 8, 6]).unwrap();
    a.plot.size(900, 600).preflight().unwrap();
}
