use std::cell::Cell;

use polars_core::prelude::*;

use crate::{kind, AlphaEncoding, AlphaValues, CategoryValues, NumericValues, Plot};

fn render<K: kind::Marker>(plot: &Plot<K>, case: &str) -> String {
    let path =
        std::env::temp_dir().join(format!("lichtung-polars-{}-{case}.svg", std::process::id()));
    plot.svg_preserve_text(&path).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    text
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "restored selected and local expansion origins are compared with their native counterparts"
)]
fn expanded_lines_preserve_actual_x_row_maps_native_alpha_declarations_and_replacement() {
    let (plot, metadata) = {
        let frame = df!("time" => [0., 5., 20.], "a" => [1., 2., 3.], "b" => [4., 5., 6.], "batch" => ["one", "one", "two"]).unwrap();
        crate::Line::from_data(&frame, "time", ["b", "a"])
            .unwrap()
            .facet_wrap("batch", 2)
            .unwrap()
            .into_parts()
    };
    let detached: crate::Line<'static> = crate::Line::from_parts(plot.clone(), metadata.clone());
    assert!(matches!(
        detached.hue("batch"),
        Err(crate::AdapterError::MissingSource { role: "hue", .. })
    ));
    // Fresh lookup must not replace the already-snapshotted x/y values.
    let frame = df!(
        "time" => [0., 5., 20.],
        "a" => [11., 12., 13.], "b" => [14., 15., 16.],
        "batch" => ["one", "one", "two"],
        "lower_b" => [3.5, 4.5, 5.5], "lower_a" => [0.5, 1.5, 2.5],
        "upper_b" => [4.5, 5.5, 6.5], "upper_a" => [1.5, 2.5, 3.5],
    )
    .unwrap();
    let calls = Cell::new(0);
    let selector = df!("variable" => ["b", "a"], "quality" => [0.25, 0.75]).unwrap();
    let adapted = crate::Line::from_parts_with_data(plot, metadata, &frame)
        .band(["lower_b", "lower_a"], ["upper_b", "upper_a"])
        .unwrap()
        .group_columns()
        .unwrap()
        .point_alpha_map_categories("batch", |values| {
            AlphaValues::categorical(values, |_| {
                calls.set(calls.get() + 1);
                160
            })
        })
        .unwrap()
        .line_alpha_columns_map_from_data(
            &selector,
            "variable",
            [] as [(&str, &str); 0],
            "quality",
            |values| AlphaValues::linear(values, (0., 1.), (80, 240)),
        )
        .unwrap()
        .facet_scales(crate::FacetScales::FreeY)
        .unwrap();
    drop(selector);
    assert_eq!(calls.get(), 2);
    assert!(adapted.clone().band("a", "b").is_err());
    assert!(crate::Line::from_data(&frame, "time", ["a", "a"]).is_err());
    let categories = CategoryValues::new("Variable", ["b", "a", "b", "a", "b", "a"]).unwrap();
    let alpha = AlphaValues::categorical(
        CategoryValues::new("batch", ["one", "one", "two"]).unwrap(),
        |_| 160,
    )
    .unwrap()
    .select([0, 0, 1, 1, 2, 2])
    .unwrap();
    let native = Plot::line(
        NumericValues::new("time", [0., 0., 5., 5., 20., 20.]),
        NumericValues::new("y", [4., 1., 5., 2., 6., 3.]),
    )
    .hue(categories.clone())
    .group(categories)
    .band(crate::Bounds::new(
        NumericValues::new("lower", [3.5, 0.5, 4.5, 1.5, 5.5, 2.5]),
        NumericValues::new("upper", [4.5, 1.5, 5.5, 2.5, 6.5, 3.5]),
    ))
    .point_alpha(AlphaEncoding::observations(alpha))
    .line_alpha(
        AlphaEncoding::keyed(
            vec![CategoryValues::new("Variable", ["b", "a"]).unwrap()],
            AlphaValues::linear(
                NumericValues::new("quality", [0.25, 0.75]),
                (0., 1.),
                (80, 240),
            )
            .unwrap(),
        )
        .unwrap(),
    )
    .facet(
        crate::FacetSpec::wrap(
            CategoryValues::new("batch", ["one", "one", "one", "one", "two", "two"]).unwrap(),
            2,
        )
        .unwrap()
        .scales(crate::FacetScales::FreeY),
    );
    assert_eq!(
        render(&adapted.into(), "wide"),
        render(&native, "wide-native")
    );
    assert_eq!(calls.get(), 2);

    let first = Series::new("duplicate".into(), [1., 2., 3.]);
    let second = Series::new("duplicate".into(), [4., 5., 6.]);
    let direct = crate::Line::new(frame.column("time").unwrap(), [&first, &second]).unwrap();
    let (plot, metadata) = direct.into_parts();
    drop(first);
    drop(second);
    let direct = crate::Line::from_parts(plot, metadata);
    // Distinct local identities survive binding; native display-collision validation is not bypassed.
    let path = std::env::temp_dir().join(format!(
        "lichtung-polars-{}-collision.svg",
        std::process::id()
    ));
    assert!(direct.clone().svg(&path).is_err());
    let renamed = direct.column_labels(["First", "Second"]).unwrap();
    let text = render(&renamed.into(), "renamed");
    assert!(text.contains("First") && text.contains("Second"));
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "projected grid dimensions, provenance and restored labels share one native comparison"
)]
fn rectangular_bindings_validate_dimensions_and_never_repeat_row_ids() {
    let (plot, metadata) = {
        let grid = df!(
            "raw_left" => [Some(1.), None],
            "raw_middle" => [Some(3.), Some(4.)],
            "raw_right" => [Some(5.), Some(6.)],
        )
        .unwrap();
        let names = ["raw_left", "raw_middle", "raw_right", "unimported"];
        let projection = crate::ColumnProjection::new(2, &names, grid.columns()).unwrap();
        crate::Heatmap::new(&projection).unwrap().into_parts()
    };
    let grid = crate::Heatmap::from_parts(plot, metadata);
    let wrong = df!("a" => [true, false, true], "b" => [false, true, false]).unwrap();
    let names = ["a", "b", "c", "unimported"];
    let wrong = crate::ColumnProjection::new(3, &names, wrong.columns()).unwrap();
    match grid.clone().heatmap_highlight(&wrong) {
        Err(crate::AdapterError::Binding {
            role: "heatmap highlights",
            message,
        }) => {
            assert!(message.contains("raw_left"));
            assert!(message.contains("raw_middle"));
            assert!(message.contains("raw_right"));
        }
        _ => panic!("the restored grid must reject a transposed cell shape"),
    }
    let wrong_ids = df!("a" => [1u32, 3, 5], "b" => [2u32, 4, 6]).unwrap();
    let wrong_ids = crate::ColumnProjection::new(3, &names, wrong_ids.columns()).unwrap();
    assert!(matches!(
        grid.clone().source_rows(&wrong_ids),
        Err(crate::AdapterError::Binding {
            role: "source rows",
            ..
        })
    ));
    assert!(grid
        .clone()
        .source_rows(&Series::new("id".into(), [1u32, 2]))
        .is_err());
    let highlight = df!("a" => [true, false], "b" => [true, false], "c" => [false, true]).unwrap();
    let ids = df!("a" => [1u32, 4], "b" => [2u32, 5], "c" => [3u32, 6]).unwrap();
    let highlight = crate::ColumnProjection::new(2, &names, highlight.columns()).unwrap();
    let ids = crate::ColumnProjection::new(2, &names, ids.columns()).unwrap();
    assert_eq!(
        crate::SourceRowsInput::bind_source_rows_repeated(&ids, None, "source rows", 3).unwrap(),
        crate::SourceRows::new([1, 2, 3, 4, 5, 6]).unwrap()
    );
    let plot = grid
        .heatmap_highlight(&highlight)
        .unwrap()
        .source_rows(&ids)
        .unwrap()
        .x_labels(["A", "B", "C"])
        .unwrap()
        .y_labels(["Old", "Other"])
        .unwrap()
        .labels(crate::Labels::raw().alias("Value", "Cell intensity"))
        .x_labels(["A", "B", "C"])
        .unwrap();
    let (plot, metadata) = plot.into_parts();
    // Two lookup rows are not the six native cell slots; restoration is infallible.
    let labels = df!("display" => ["Top", "Bottom"]).unwrap();
    let plot = crate::Heatmap::from_parts_with_data(plot, metadata, &labels)
        .row_labels("display")
        .unwrap();
    let native = Plot::heatmap(
        CategoryValues::new(
            "Column",
            [
                "__lichtung_grid_x_0",
                "__lichtung_grid_x_1",
                "__lichtung_grid_x_2",
                "__lichtung_grid_x_0",
                "__lichtung_grid_x_1",
                "__lichtung_grid_x_2",
            ],
        )
        .unwrap(),
        CategoryValues::new(
            "Row",
            [
                "__lichtung_grid_y_0",
                "__lichtung_grid_y_0",
                "__lichtung_grid_y_0",
                "__lichtung_grid_y_1",
                "__lichtung_grid_y_1",
                "__lichtung_grid_y_1",
            ],
        )
        .unwrap(),
        NumericValues::nullable(
            "Value",
            [Some(1.), Some(3.), Some(5.), None, Some(4.), Some(6.)],
        ),
    )
    .heatmap_highlight(crate::BooleanValues::new(
        "Highlights",
        [true, true, false, false, false, true],
    ))
    .source_rows(crate::SourceRows::new([1, 2, 3, 4, 5, 6]).unwrap())
    .labels(
        crate::Labels::raw()
            .alias("Value", "Cell intensity")
            .aliases([
                ("__lichtung_grid_x_0", "A"),
                ("__lichtung_grid_x_1", "B"),
                ("__lichtung_grid_x_2", "C"),
                ("__lichtung_grid_y_0", "Top"),
                ("__lichtung_grid_y_1", "Bottom"),
            ]),
    );
    assert_eq!(render(&plot.into(), "grid"), render(&native, "grid-native"));
}
