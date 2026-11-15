use polars_core::prelude::*;

use crate::{
    kind, AdapterError, AlphaEncoding, AlphaValues, BooleanInput, CategoryValues, ColumnProjection,
    ColumnsInput, Line, NumericValues, Plot, SourceRows, SourceRowsInput,
};

fn render<K: kind::Marker>(plot: &Plot<K>, case: &str) -> String {
    let path = std::env::temp_dir().join(format!(
        "lichtung-polars-projection-{}-{case}.svg",
        std::process::id()
    ));
    plot.svg_preserve_text(&path).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    text
}

fn column_alpha(field: &str, keys: [&str; 2]) -> AlphaEncoding {
    AlphaEncoding::keyed(
        vec![CategoryValues::new(field, keys).unwrap()],
        AlphaValues::new("opacity", [100, 200]).unwrap(),
    )
    .unwrap()
}

#[test]
fn descriptor_checks_only_selection_structure() {
    let columns = [Column::new("a".into(), [1., 2.])];
    assert!(matches!(
        ColumnProjection::new(2, &["a"], &[]),
        Err(AdapterError::Binding {
            role: "column projection",
            ..
        })
    ));
    assert!(matches!(
        ColumnProjection::new(2, &["a", "a"], &columns),
        Err(AdapterError::Binding {
            role: "column projection",
            ..
        })
    ));
    let duplicate = [columns[0].clone(), columns[0].clone()];
    assert!(matches!(
        ColumnProjection::new(2, &["a"], &duplicate),
        Err(AdapterError::Binding {
            role: "column projection",
            ..
        })
    ));
    assert!(matches!(
        ColumnProjection::new(2, &["other"], &columns),
        Err(AdapterError::Binding {
            role: "column projection",
            ..
        })
    ));
    assert!(matches!(
        ColumnProjection::new(3, &["a"], &columns),
        Err(AdapterError::Length {
            expected: 3,
            actual: 2,
            ..
        })
    ));
    assert!(matches!(
        ColumnProjection::from_direct(&[], &[]),
        Err(AdapterError::Binding {
            role: "column projection",
            ..
        })
    ));
    assert!(matches!(
        ColumnProjection::from_direct(&["unused", "unused"], &columns),
        Err(AdapterError::Binding {
            role: "column projection",
            ..
        })
    ));
}

#[test]
fn numeric_discovery_covers_concrete_primitives_without_observation_or_storage_inference() {
    let primitive = [
        ("u8", DataType::UInt8),
        ("u16", DataType::UInt16),
        ("u32", DataType::UInt32),
        ("u64", DataType::UInt64),
        ("u128", DataType::UInt128),
        ("i8", DataType::Int8),
        ("i16", DataType::Int16),
        ("i32", DataType::Int32),
        ("i64", DataType::Int64),
        ("i128", DataType::Int128),
        ("f16", DataType::Float16),
        ("f32", DataType::Float32),
        ("f64", DataType::Float64),
    ];
    let categories = Categories::new(
        "projection categories".into(),
        "lichtung-plot tests".into(),
        CategoricalPhysical::U8,
    );
    let other = [
        ("boolean", DataType::Boolean),
        ("null", DataType::Null),
        ("date", DataType::Date),
        ("string", DataType::String),
        ("list", DataType::List(Box::new(DataType::Int64))),
        ("category", DataType::from_categories(categories)),
        (
            "enum",
            DataType::from_frozen_categories(FrozenCategories::new(["a"]).unwrap()),
        ),
    ];
    let frame = DataFrame::new_infer_height(
        primitive
            .into_iter()
            .chain(other)
            .map(|(name, dtype)| Column::new_empty(name.into(), &dtype))
            .collect(),
    )
    .unwrap();
    assert_eq!(
        ColumnProjection::numeric_names(&frame),
        [
            "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "f16", "f32",
            "f64"
        ]
    );
    let all_null = df!(
        "typed_missing" => [None::<f64>, None],
        "source_id" => [1u64, 2], "flag" => [true, false],
    )
    .unwrap();
    assert_eq!(
        ColumnProjection::numeric_names(&all_null),
        ["typed_missing", "source_id"]
    );
}

#[test]
fn projected_line_orders_keep_the_union_of_names_and_release_projection_borrows() {
    let candidates = df!("b" => [4., 5., 6.], "a" => [1., 2., 3.]).unwrap();
    let main = df!("Variable__" => [0.; 3]).unwrap();
    let x = Series::new("Variable___".into(), [0., 1., 2.]);
    let selected = {
        // Variable/Variable_ represent unimported Decimal/Struct fields: names
        // only, with no fabricated dtype or placeholder Column.
        let names = ["Variable", "b", "Variable_", "a"];
        let columns = [
            candidates.column("a").unwrap().clone(),
            candidates.column("b").unwrap().clone(),
        ];
        let projection = ColumnProjection::new(3, &names, &columns).unwrap();
        assert_eq!(
            (&projection)
                .bind_columns(None)
                .unwrap()
                .iter()
                .map(|column| column.name().as_str())
                .collect::<Vec<_>>(),
            ["a", "b"]
        );
        Line::from_data(&main, &x, &projection)
            .unwrap()
            .line_alpha(column_alpha("Variable____", ["a", "b"]))
            .unwrap()
            .labels(crate::Labels::raw())
    };
    let native = Plot::line(
        NumericValues::new("Variable___", [0., 0., 1., 1., 2., 2.]),
        NumericValues::new("y", [1., 4., 2., 5., 3., 6.]),
    )
    .hue(CategoryValues::new("Variable____", ["a", "b", "a", "b", "a", "b"]).unwrap())
    .group(CategoryValues::new("Variable____", ["a", "b", "a", "b", "a", "b"]).unwrap())
    .line_alpha(column_alpha("Variable____", ["a", "b"]))
    .labels(crate::Labels::raw());
    assert_eq!(
        render(&selected.into(), "line-explicit"),
        render(&native, "line-native")
    );

    let names = ["Variable", "b", "Variable_", "a"];
    let discovered = ColumnProjection::numeric_names(&candidates);
    assert_eq!(discovered, ["b", "a"]);
    let columns = candidates.select(discovered).unwrap();
    let projection = ColumnProjection::new(3, &names, columns.columns()).unwrap();
    assert_eq!(
        (&projection)
            .bind_columns(None)
            .unwrap()
            .iter()
            .map(|column| column.name().as_str())
            .collect::<Vec<_>>(),
        ["b", "a"]
    );
    let indexed: Plot<kind::Line> = Line::indexed(&projection)
        .unwrap()
        .line_alpha(column_alpha("Variable__", ["b", "a"]))
        .unwrap()
        .into();
    indexed.preflight().unwrap();
}

#[test]
fn projected_distribution_modes_avoid_unimported_names() {
    let names = ["Variable", "a", "Variable_", "b"];
    let columns = [
        Column::new("a".into(), [1., 2., 3.]),
        Column::new("b".into(), [4., 5., 6.]),
    ];
    let projection = ColumnProjection::new(3, &names, &columns).unwrap();
    let category = Series::new("cohort".into(), ["one"; 3]);
    let samples: Plot<kind::Boxplot> = crate::Boxplot::new(&category, &projection)
        .unwrap()
        .fill_alpha(
            AlphaEncoding::keyed(
                vec![
                    CategoryValues::new("cohort", ["one", "one"]).unwrap(),
                    CategoryValues::new("Variable__", ["a", "b"]).unwrap(),
                ],
                AlphaValues::new("opacity", [100, 200]).unwrap(),
            )
            .unwrap(),
        )
        .unwrap()
        .into();
    samples.preflight().unwrap();
    let by_column: Plot<kind::Boxplot> = crate::Boxplot::columns(&projection)
        .unwrap()
        .fill_alpha(column_alpha("Variable__", ["a", "b"]))
        .unwrap()
        .into();
    by_column.preflight().unwrap();
}

#[test]
fn empty_selected_names_keep_raw_groups_and_native_validation_boundaries() {
    let frame =
        df!("time" => [0., 1., 2.], "" => [1., 2., 3.], "Column 1" => [4., 5., 6.]).unwrap();
    let selected = Line::from_data(&frame, "time", ["", "Column 1"])
        .unwrap()
        .labels(crate::Labels::raw())
        .column_labels(["Empty response", "Named response"])
        .unwrap();
    let names = ["time", "", "Column 1"];
    let columns = [
        frame.column("").unwrap().clone(),
        frame.column("Column 1").unwrap().clone(),
    ];
    let projection = ColumnProjection::new(3, &names, &columns).unwrap();
    let projected = Line::new(frame.column("time").unwrap(), &projection)
        .unwrap()
        .labels(crate::Labels::raw())
        .column_labels(["Empty response", "Named response"])
        .unwrap();
    let categories =
        CategoryValues::new("Variable", ["", "Column 1", "", "Column 1", "", "Column 1"]).unwrap();
    let native = Plot::line(
        NumericValues::new("time", [0., 0., 1., 1., 2., 2.]),
        NumericValues::new("y", [1., 4., 2., 5., 3., 6.]),
    )
    .hue(categories.clone())
    .group(categories)
    .labels(crate::Labels::raw().aliases([("", "Empty response"), ("Column 1", "Named response")]));
    let expected = render(&native, "empty-native");
    assert_eq!(render(&selected.into(), "empty-selected"), expected);
    assert_eq!(render(&projected.into(), "empty-projected"), expected);

    let local: Plot<kind::Line> = Line::new(
        frame.column("time").unwrap(),
        [frame.column("").unwrap(), frame.column("Column 1").unwrap()],
    )
    .unwrap()
    .into();
    assert!(matches!(
        local.preflight(),
        Err(crate::Error::Render(detail)) if detail.contains("distinct raw identities")
    ));
    let matrix: Plot<kind::Matrix> = crate::Matrix::new(&projection).unwrap().into();
    assert!(matches!(matrix.preflight(), Err(crate::Error::Data(_))));
}

#[test]
fn one_column_collection_and_zero_rows_keep_existing_wide_meaning() {
    let x = Series::new("time".into(), [0., 1.]);
    let columns = [Column::new("only".into(), [2., 3.])];
    let projection = ColumnProjection::new(2, &["only"], &columns).unwrap();
    let line = Line::new(&x, &projection).unwrap().group_columns().unwrap();
    assert!(line.band(&columns[0], &columns[0]).is_err());
    let direct = ColumnProjection::from_direct(&[], &columns).unwrap();
    let line = Line::new(&x, &direct).unwrap().group_columns().unwrap();
    assert!(matches!(
        line.band(&columns[0], &columns[0]),
        Err(AdapterError::Binding { role: "band", .. })
    ));

    let empty_columns = [Column::new_empty("only".into(), &DataType::Float64)];
    let projection = ColumnProjection::new(0, &["only"], &empty_columns).unwrap();
    let empty: Plot<kind::Line> = Line::indexed(&projection)
        .unwrap()
        .group_columns()
        .unwrap()
        .into();
    assert!(matches!(empty.preflight(), Err(crate::Error::Data(_))));
    let direct = ColumnProjection::from_direct(&[], &empty_columns).unwrap();
    let empty: Plot<kind::Line> = Line::indexed(&direct)
        .unwrap()
        .group_columns()
        .unwrap()
        .into();
    assert!(matches!(empty.preflight(), Err(crate::Error::Data(_))));
}

#[test]
fn projected_roles_keep_dtype_admission_and_y_before_x_error_order() {
    let columns = [Column::new("flag".into(), [true, false])];
    let projection = ColumnProjection::new(2, &["flag"], &columns).unwrap();
    assert!(matches!(
        Line::new("absent", &projection),
        Err(AdapterError::UnsupportedDtype {
            channel: "y",
            dtype: DataType::Boolean,
            ..
        })
    ));
    let columns = [Column::new("value".into(), [1., 2.])];
    let projection = ColumnProjection::new(2, &["value"], &columns).unwrap();
    assert!(matches!(
        Line::new("absent", &projection),
        Err(AdapterError::MissingSource { role: "x", .. })
    ));

    // Factory admission must not prescan the later ragged column before the
    // numeric consumer rejects this first Boolean column, or attempt x lookup.
    let columns = [
        Column::new("flag".into(), [true, false]),
        Column::new("long".into(), [1., 2., 3.]),
    ];
    let direct = ColumnProjection::from_direct(&[], &columns).unwrap();
    assert!(matches!(
        Line::new("absent", &direct),
        Err(AdapterError::UnsupportedDtype {
            name,
            channel: "y",
            dtype: DataType::Boolean,
        }) if name == "flag"
    ));
    // Within the next column, its length still precedes its dtype admission.
    let columns = [
        Column::new("value".into(), [1., 2.]),
        Column::new("flag".into(), [true, false, true]),
    ];
    let direct = ColumnProjection::from_direct(&[], &columns).unwrap();
    assert!(matches!(
        Line::new("absent", &direct),
        Err(AdapterError::Length {
            name,
            role: "y",
            expected: 2,
            actual: 3,
        }) if name == "flag"
    ));
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one direct projection witnesses namespace/local identity and independent cell authority without another rendering matrix"
)]
fn direct_namespace_keeps_local_ordinals_and_cell_authorities() {
    let names = ["Variable", "Variable_"];
    let columns = [
        Column::new("same".into(), [1., 2.]),
        Column::new("same".into(), [3., 4.]),
        Column::new("".into(), [5., 6.]),
    ];
    let direct = ColumnProjection::from_direct(&names, &columns).unwrap();
    let keys = [
        "__lichtung_variable_0",
        "__lichtung_variable_1",
        "__lichtung_variable_2",
    ];
    let alpha = AlphaEncoding::keyed(
        [CategoryValues::new("Variable__", keys).unwrap()],
        AlphaValues::new("opacity", [80, 160, 240]).unwrap(),
    )
    .unwrap();
    let x = Series::new("time".into(), [0., 1.]);
    for line in [
        Line::new(&x, &direct).unwrap(),
        Line::indexed(&direct).unwrap(),
    ] {
        let line = line.line_alpha(alpha.clone()).unwrap();
        let (native, _metadata) = line.clone().into_parts();
        assert_eq!(
            native
                .get_labels()
                .get_aliases()
                .unwrap()
                .collect::<Vec<_>>(),
            [(keys[0], "same"), (keys[1], "same"), (keys[2], "Column 3")]
        );
        assert!(matches!(native.preflight(), Err(crate::Error::Render(_))));
        let renamed: Plot<kind::Line> = line
            .column_labels(["First", "Second", "Third"])
            .unwrap()
            .into();
        // Keyed alpha also witnesses the generated field being Variable__.
        renamed.preflight().unwrap();
    }

    // Retained lookup height is deliberately three, not the direct grid's two.
    let lookup = df!("Variable" => [0.; 3], "Variable_" => [0.; 3]).unwrap();
    let (plot, metadata) = crate::Heatmap::new(&direct)
        .unwrap()
        .x_labels(["A", "B", "C"])
        .unwrap()
        .into_parts();
    let grid = crate::Heatmap::from_parts_with_data(plot, metadata, &lookup);
    let transposed = [
        Column::new("flag".into(), [true, false, true]),
        Column::new("flag".into(), [false, true, false]),
    ];
    let transposed = ColumnProjection::from_direct(&names, &transposed).unwrap();
    assert!(matches!(
        grid.clone().heatmap_highlight(&transposed),
        Err(AdapterError::Binding {
            role: "heatmap highlights",
            ..
        })
    ));
    let transposed_ids = [
        Column::new("id".into(), [101u32, 103, 105]),
        Column::new("id".into(), [102u32, 104, 106]),
    ];
    let transposed_ids = ColumnProjection::from_direct(&names, &transposed_ids).unwrap();
    assert!(matches!(
        grid.clone().source_rows(&transposed_ids),
        Err(AdapterError::Binding {
            role: "source rows",
            ..
        })
    ));

    let highlights = [
        Column::new("flag".into(), [Some(true), Some(false)]),
        Column::new("flag".into(), [None, Some(true)]),
        Column::new("".into(), [Some(false), Some(false)]),
    ];
    let highlights = ColumnProjection::from_direct(&names, &highlights).unwrap();
    assert_eq!(
        (&highlights)
            .bind_boolean_repeated(Some(&lookup), "heatmap highlights", 3)
            .unwrap()
            .len(),
        6
    );
    let ids = [
        Column::new("id".into(), [101u32, 104]),
        Column::new("id".into(), [102u32, 105]),
        Column::new("".into(), [103u32, 106]),
    ];
    let ids = ColumnProjection::from_direct(&names, &ids).unwrap();
    assert_eq!(
        (&ids)
            .bind_source_rows_repeated(Some(&lookup), "source rows", 3)
            .unwrap(),
        SourceRows::new([101, 102, 103, 104, 105, 106]).unwrap()
    );
    let native: Plot<kind::Heatmap> = grid
        .heatmap_highlight(&highlights)
        .unwrap()
        .source_rows(&ids)
        .unwrap()
        .into();
    // The nullable Boolean is row 0 / column 1: row-major ID 102, not 103.
    assert!(matches!(
        native.preflight(),
        Err(crate::Error::Data(detail))
            if detail.contains("null") && detail.contains("source row 102")
    ));
}
