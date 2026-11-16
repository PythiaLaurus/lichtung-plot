use polars_core::prelude::*;

use crate::{
    kind, AdapterError, AlphaEncoding, AlphaValues, BooleanInput, CategoryValues, ColumnProjection,
    Line, Plot, SourceRows, SourceRowsInput,
};

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
