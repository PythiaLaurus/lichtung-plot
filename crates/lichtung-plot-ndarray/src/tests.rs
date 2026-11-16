use crate::{snapshot, AdapterError, BooleanInput, ColumnsInput, NumericElement, SourceRowsInput};
use ndarray::{array, s, Array2, ShapeBuilder};
use std::cell::Cell;
use std::error::Error as _;

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
    assert!(values[2].unwrap().is_infinite());
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
    let view = transpose.slice(s![..;-1,..;2]);
    let validity = array![[true, false, true, true], [false, true, true, false]];
    let (_, columns) = (view, validity.t()).bind_columns(None).unwrap();
    assert_eq!(columns[0].1, [Some(9.), None, Some(3.), Some(0.)]);
    assert_eq!(columns[1].1, [None, Some(8.), Some(5.), None]);
    let broadcast = array![1, 2, 3];
    let view = broadcast.broadcast((2, 3)).unwrap();
    let (_, columns) = view.bind_columns(None).unwrap();
    assert_eq!(columns[2].1, [Some(3.), Some(3.)]);
    let highlights = array![[Some(true), None], [Some(false), Some(true)]];
    assert_eq!(
        [1, 0]
            .raw_boolean(Some(&highlights.view()), "highlights")
            .unwrap()
            .1,
        [None, Some(true), Some(true), Some(false)]
    );
}
#[test]
fn provenance_uses_exact_ids_and_reports_logical_slots() {
    let ids = array![0_u128, 9_007_199_254_740_993];
    if usize::BITS > 53 {
        assert_eq!(
            crate::input::ids(ids.view(), None, "IDs").unwrap()[1],
            usize::try_from(9_007_199_254_740_993_u128).unwrap()
        );
    }
    let invalid = array![Some(0_i128), None, Some(-1)];
    assert!(matches!(
        invalid.view().bind_source_rows(None, "IDs"),
        Err(AdapterError::SourceRow { slot: 1, .. })
    ));
    assert!(matches!(
        invalid.slice(s![..;-1]).bind_source_rows(None, "IDs"),
        Err(AdapterError::SourceRow { slot: 0, .. })
    ));
    assert!(matches!(
        array![u128::MAX].view().bind_source_rows(None, "IDs"),
        Err(AdapterError::SourceRow { slot: 0, .. })
    ));
    let duplicate = array![4, 4]
        .view()
        .bind_source_rows(None, "IDs")
        .unwrap_err();
    assert!(duplicate.source().is_some());
    assert!(duplicate.to_string().contains("duplicate 4"));
    let source = array![[1, 2, 101, 102], [3, 4, 103, 104]];
    assert_eq!(
        [3, 2]
            .raw_source_rows(Some(&source.view()), "cell IDs")
            .unwrap(),
        [102, 101, 104, 103]
    );
    let (plot, metadata) = crate::Heatmap::from_data(source.view(), [0, 1])
        .unwrap()
        .into_parts();
    assert!(
        crate::Heatmap::from_parts_with_data(plot, metadata, source.view())
            .unwrap()
            .source_rows([3, 2])
            .is_ok()
    );
    assert!(crate::Heatmap::from_data(source.view(), [0, 1])
        .unwrap()
        .source_rows([2])
        .is_err());
    assert!([2, 2]
        .bind_source_rows(Some(&source.view()), "cell IDs")
        .is_err());
    let invalid_column = array![[4, -1]];
    assert!(matches!(
        [0, 1].raw_source_rows(Some(&invalid_column.view()), "cell IDs"),
        Err(AdapterError::SourceRow {
            slot: 0,
            column: Some(1),
            ..
        })
    ));
}

pub(crate) fn render<K: crate::kind::Marker>(plot: &crate::Plot<K>, case: &str) -> String {
    let path = std::env::temp_dir().join(format!(
        "lichtung-ndarray-{}-{case}.svg",
        std::process::id()
    ));
    plot.svg_preserve_text(&path).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    text
}
#[test]
fn wide_views_preserve_actual_coordinates_selected_order_bounds_and_mapping_callbacks() {
    let source = array![[0., 1., 4.], [5., 2., 5.], [20., 3., 6.]];
    let present = Array2::from_elem(source.dim(), true);
    let batches = array!["A", "A", "B"];
    let calls = Cell::new(0);
    let lower = array![[3.5, 0.5], [4.5, 1.5], [5.5, 2.5]];
    let upper = &lower + 1.;
    let x = (source.slice(s![.., 0..1]), present.slice(s![.., 0..1]));
    let y1 = (source.slice(s![.., 1..2]), present.slice(s![.., 1..2]));
    let y2 = (source.slice(s![.., 2..3]), present.slice(s![.., 2..3]));
    let entries: [(usize, &dyn crate::ArraySource); 3] = [(1, &y1), (0, &x), (2, &y2)];
    let projected = crate::ColumnProjection::new(source.dim(), &entries).unwrap();
    let (plot, metadata) = crate::Line::from_data(projected, 0, [2, 1])
        .unwrap()
        .into_parts();
    let adapted = crate::Line::from_parts(plot, metadata)
        .group_columns()
        .unwrap()
        .band(lower.view(), upper.view())
        .unwrap()
        .point_alpha_map_categories(batches.view(), |values| {
            crate::AlphaValues::categorical(values, |_| {
                calls.set(calls.get() + 1);
                180
            })
        })
        .unwrap()
        .column_labels(["B", "A"])
        .unwrap();
    assert_eq!(calls.get(), 2);
    let groups = crate::CategoryValues::new(
        "Variable",
        [
            "Column 3", "Column 2", "Column 3", "Column 2", "Column 3", "Column 2",
        ],
    )
    .unwrap();
    let native = crate::Plot::line(
        crate::NumericValues::new("Column 1", [0., 0., 5., 5., 20., 20.]),
        crate::NumericValues::new("y", [4., 1., 5., 2., 6., 3.]),
    )
    .hue(groups.clone())
    .group(groups)
    .band(crate::Bounds::new(
        crate::NumericValues::new("lower", [3.5, 0.5, 4.5, 1.5, 5.5, 2.5]),
        crate::NumericValues::new("upper", [4.5, 1.5, 5.5, 2.5, 6.5, 3.5]),
    ))
    .point_alpha(crate::AlphaEncoding::observations(
        crate::AlphaValues::categorical(
            crate::CategoryValues::new("alpha map", ["A", "A", "B"]).unwrap(),
            |_| 180,
        )
        .unwrap()
        .select([0, 0, 1, 1, 2, 2])
        .unwrap(),
    ))
    .labels(crate::Labels::default().aliases([("Column 3", "B"), ("Column 2", "A")]));
    let expected = render(&native, "wide-native");
    assert_eq!(render(&adapted.into(), "wide"), expected);
    // Direct matrix ordinals must not alias a separately selected source x column.
    let direct = crate::Line::from_data(source.view(), 0, source.slice(s![.., 1..;-1]))
        .unwrap()
        .band(lower.view(), upper.view())
        .unwrap()
        .point_alpha_map_categories(batches.view(), |values| {
            crate::AlphaValues::categorical(values, |_| 180)
        })
        .unwrap()
        .column_labels(["B", "A"])
        .unwrap();
    assert_eq!(render(&direct.into(), "wide-direct"), expected);
    assert_eq!(calls.get(), 2);
    assert!(crate::Line::from_data(source.view(), 0, [1, 2])
        .unwrap()
        .band(1, 2)
        .is_err());
    assert!(crate::Line::from_data(source.view(), 0, [1, 2])
        .unwrap()
        .source_rows(array![1, 2, 3].view())
        .is_err());
}
