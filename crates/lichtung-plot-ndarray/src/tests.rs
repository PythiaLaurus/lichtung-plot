use std::error::Error as _;

use ndarray::{array, s, Array2, ShapeBuilder};

use crate::{
    columns, grid, provenance, snapshot, BooleanElement, CategoryElement, Error, NumericElement,
};

#[test]
fn numeric_snapshot_retains_rounding_null_and_nonfinite_identity() {
    let integers = array![Some(9_007_199_254_740_993_u64), None, Some(u64::MAX)];
    let rounded = snapshot::values(integers.view(), None, NumericElement::to_f64, "test").unwrap();
    assert_eq!(
        rounded
            .iter()
            .map(|value| value.map(f64::to_bits))
            .collect::<Vec<_>>(),
        [
            Some(9_007_199_254_740_992.0_f64.to_bits()),
            None,
            Some(18_446_744_073_709_551_616.0_f64.to_bits())
        ]
    );
    let floating = array![Some(-0.0), Some(f64::NAN), Some(f64::INFINITY), None];
    let values = snapshot::values(floating.view(), None, NumericElement::to_f64, "test").unwrap();
    assert_eq!(values[0].unwrap().to_bits(), (-0.0_f64).to_bits());
    assert!(values[1].unwrap().is_nan());
    assert_eq!(values[2].unwrap().to_bits(), f64::INFINITY.to_bits());
    assert!(values[3].is_none());
    let extreme = array![i128::MIN, i128::MAX];
    let values = snapshot::values(extreme.view(), None, NumericElement::to_f64, "test").unwrap();
    assert_eq!(values[0].unwrap().to_bits(), (-2.0_f64.powi(127)).to_bits());
    assert_eq!(values[1].unwrap().to_bits(), 2.0_f64.powi(127).to_bits());
}

#[test]
fn grid_pairs_logical_cells_with_independently_strided_validity() {
    let matrix = Array2::from_shape_vec((3, 4).f(), (0..12).collect()).unwrap();
    let transpose = matrix.t();
    let view = transpose.slice(s![..;-1, ..;2]);
    let validity = array![[true, false, true, true], [false, true, true, false]];
    let cells = grid::cells(
        view,
        &["A", "B"],
        &["late", "middle", "early", "baseline"],
        Some(validity.t()),
    )
    .unwrap();
    assert_eq!(cells.x, ["A", "B", "A", "B", "A", "B", "A", "B"]);
    assert_eq!(
        cells.y,
        ["late", "late", "middle", "middle", "early", "early", "baseline", "baseline"]
    );
    assert_eq!(
        cells
            .fill
            .iter()
            .map(|value| value.map(f64::to_bits))
            .collect::<Vec<_>>(),
        [
            Some(9.0_f64.to_bits()),
            None,
            None,
            Some(8.0_f64.to_bits()),
            Some(3.0_f64.to_bits()),
            Some(5.0_f64.to_bits()),
            Some(0.0_f64.to_bits()),
            None
        ]
    );
}

#[test]
fn table_columns_keep_explicit_variable_roles_and_empty_shapes() {
    let values = array![[1, 2, 3], [10, 20, 30]];
    let columns = columns::columns(&["raw name", "other"], values.t(), None).unwrap();
    assert_eq!(
        columns.iter().map(|column| column.name).collect::<Vec<_>>(),
        ["raw name", "other"]
    );
    assert_eq!(
        columns[0]
            .values
            .iter()
            .map(|value| value.unwrap().to_bits())
            .collect::<Vec<_>>(),
        [1.0_f64.to_bits(), 2.0_f64.to_bits(), 3.0_f64.to_bits()]
    );
    assert_eq!(
        columns[1]
            .values
            .iter()
            .map(|value| value.unwrap().to_bits())
            .collect::<Vec<_>>(),
        [10.0_f64.to_bits(), 20.0_f64.to_bits(), 30.0_f64.to_bits()]
    );
    assert!(matches!(
        crate::numeric_columns(&["too few"], values.t()),
        Err(Error::ColumnNames {
            expected: 2,
            actual: 1
        })
    ));
    let no_rows = Array2::<f64>::zeros((0, 2));
    assert!(crate::numeric_columns(&["same", "same"], no_rows.view())
        .unwrap()
        .iter()
        .all(lichtung_plot::NumericValues::is_empty));
    assert!(
        crate::numeric_columns(&[], Array2::<f64>::zeros((2, 0)).view())
            .unwrap()
            .is_empty()
    );
}

#[test]
fn category_snapshot_owns_exact_nullable_raw_keys() {
    let mut keys = array![
        Some(" A ".to_owned()),
        None,
        Some("é".to_owned()),
        Some("e\u{301}".to_owned())
    ];
    let values = snapshot::values(
        keys.view(),
        None,
        |value| value.as_key().map(str::to_owned),
        "categories",
    )
    .unwrap();
    keys[0] = Some("edited".into());
    drop(keys);
    assert_eq!(
        values,
        [
            Some(" A ".into()),
            None,
            Some("é".into()),
            Some("e\u{301}".into())
        ]
    );
    let native = crate::categories("raw name", array![Some(" A "), None].view()).unwrap();
    assert_eq!(native.name(), "raw name");
    assert_eq!(native.len(), 2);
}

#[test]
fn boolean_validity_is_presence_not_selection_or_numeric_coercion() {
    let values = array![Some(true), Some(false), None, Some(true)];
    let validity = array![false, true, true, true];
    let snapshot = snapshot::values(
        values.view(),
        Some(validity.view()),
        BooleanElement::as_boolean,
        "booleans",
    )
    .unwrap();
    assert_eq!(snapshot, [None, Some(false), None, Some(true)]);
    assert_eq!(
        crate::booleans_with_validity("flag", values.view(), validity.view())
            .unwrap()
            .len(),
        4
    );
}

#[test]
fn provenance_uses_exact_ids_and_reports_logical_slots() {
    let ids = array![0_u128, 9_007_199_254_740_993];
    if usize::BITS > 53 {
        assert_eq!(
            provenance::ids(ids.view()).unwrap()[1],
            usize::try_from(9_007_199_254_740_993_u128).unwrap()
        );
    }
    let invalid = array![Some(0_i128), None, Some(-1)];
    assert!(matches!(
        crate::source_rows(invalid.view()),
        Err(Error::SourceRow { slot: 1, .. })
    ));
    assert!(matches!(
        crate::source_rows(invalid.slice(s![..;-1])),
        Err(Error::SourceRow { slot: 0, .. })
    ));
    assert!(matches!(
        crate::source_rows(array![u128::MAX].view()),
        Err(Error::SourceRow { slot: 0, .. })
    ));
    let duplicate = crate::source_rows(array![4, 4].view()).unwrap_err();
    assert!(duplicate.source().is_some());
    assert!(duplicate.to_string().contains("duplicate 4"));
}

#[test]
fn validity_requires_each_logical_dimension_not_just_length() {
    let values = Array2::<f64>::zeros((2, 3));
    let wrong = Array2::<bool>::from_elem((3, 2), true);
    assert!(
        matches!(crate::numeric_columns_with_validity(&["a", "b", "c"], values.view(), wrong.view()), Err(Error::Shape { expected, actual, .. }) if expected == [2, 3] && actual == [3, 2])
    );
    assert!(matches!(
        crate::heatmap_values_with_validity(
            "v",
            values.view(),
            ("x", &["a", "b", "c"]),
            ("y", &["r", "s"]),
            wrong.view()
        ),
        Err(Error::Shape { .. })
    ));
    assert!(matches!(
        crate::numeric_with_validity("n", array![1, 2].view(), array![true].view()),
        Err(Error::Shape { .. })
    ));
}

#[test]
fn grid_keys_identify_axes_without_fabricating_empty_domains() {
    let values = array![[1, 2]];
    assert!(matches!(
        crate::heatmap_values("v", values.view(), ("x", &["a", "a"]), ("y", &["r"])),
        Err(Error::DuplicateAxisKey { axis: "x", .. })
    ));
    assert!(matches!(
        crate::heatmap_values("v", values.view(), ("x", &["a"]), ("y", &["r"])),
        Err(Error::Shape { operation: "x", .. })
    ));
    let cells = crate::heatmap_values(
        "v",
        Array2::<f64>::zeros((0, 2)).view(),
        ("x", &[" A", "A"]),
        ("y", &[]),
    )
    .unwrap();
    assert!(cells.x.is_empty() && cells.y.is_empty() && cells.fill.is_empty());
}
