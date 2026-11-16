use ndarray::{array, s, Array2};

use crate::{
    AdapterError, ArraySource, CategoryValues, ColumnProjection, Heatmap, Labels, Line,
    NumericValues, Plot, SourceRowsInput,
};

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
