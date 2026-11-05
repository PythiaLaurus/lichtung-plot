use std::cell::Cell;

use polars_core::prelude::*;

use crate::{kind, AlphaEncoding, AlphaValues, CategoryValues, NumericValues, Plot};

fn render<K: kind::Marker>(plot: Plot<K>, case: &str) -> String {
    let path =
        std::env::temp_dir().join(format!("lichtung-polars-{}-{case}.svg", std::process::id()));
    plot.svg_preserve_text(&path).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    text
}

#[test]
fn expanded_lines_preserve_actual_x_row_maps_native_alpha_declarations_and_replacement() {
    let frame = df!("time" => [0., 5., 20.], "a" => [1., 2., 3.], "b" => [4., 5., 6.], "batch" => ["one", "one", "two"]).unwrap();
    let calls = Cell::new(0);
    let selector = df!("variable" => ["b", "a"], "quality" => [0.25, 0.75]).unwrap();
    let adapted = crate::Line::from_data(&frame, "time", ["b", "a"])
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
        .facet_wrap("batch", 2)
        .unwrap();
    drop(selector);
    assert_eq!(calls.get(), 2);
    assert!(crate::Line::from_data(&frame, "time", ["b", "a"])
        .unwrap()
        .band("a", "b")
        .is_err());
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
        .unwrap(),
    );
    assert_eq!(
        render(adapted.into(), "wide"),
        render(native, "wide-native")
    );
    assert_eq!(calls.get(), 2);

    let first = Series::new("duplicate".into(), [1., 2., 3.]);
    let second = Series::new("duplicate".into(), [4., 5., 6.]);
    let direct = crate::Line::new(frame.column("time").unwrap(), [&first, &second]).unwrap();
    // Distinct local identities survive binding; native display-collision validation is not bypassed.
    let path = std::env::temp_dir().join(format!(
        "lichtung-polars-{}-collision.svg",
        std::process::id()
    ));
    assert!(direct.clone().svg(&path).is_err());
    let renamed = direct.column_labels(["First", "Second"]).unwrap();
    let text = render(renamed.into(), "renamed");
    assert!(text.contains("First") && text.contains("Second"));
}

#[test]
fn rectangular_bindings_validate_dimensions_and_never_repeat_row_ids() {
    let grid =
        df!("a" => [Some(1.), None], "b" => [Some(3.), Some(4.)], "c" => [Some(5.), Some(6.)])
            .unwrap();
    let wrong = df!("a" => [true, false, true], "b" => [false, true, false]).unwrap();
    assert!(crate::Heatmap::new(&grid)
        .unwrap()
        .heatmap_highlight(&wrong)
        .is_err());
    assert!(crate::Heatmap::new(&grid)
        .unwrap()
        .source_rows(&Series::new("id".into(), [1u32, 2]))
        .is_err());
    let highlight = df!("a" => [true, false], "b" => [false, false], "c" => [false, true]).unwrap();
    let ids = df!("a" => [1u32, 4], "b" => [2u32, 5], "c" => [3u32, 6]).unwrap();
    let plot = crate::Heatmap::new(&grid)
        .unwrap()
        .heatmap_highlight(&highlight)
        .unwrap()
        .source_rows(&ids)
        .unwrap()
        .x_labels(["A", "B", "C"])
        .unwrap()
        .y_labels(["Old", "Other"])
        .unwrap()
        .labels(crate::Labels::raw())
        .x_labels(["A", "B", "C"])
        .unwrap()
        .y_labels(["Top", "Bottom"])
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
        [true, false, false, false, false, true],
    ))
    .source_rows(crate::SourceRows::new([1, 2, 3, 4, 5, 6]).unwrap())
    .labels(crate::Labels::raw().aliases([
        ("__lichtung_grid_x_0", "A"),
        ("__lichtung_grid_x_1", "B"),
        ("__lichtung_grid_x_2", "C"),
        ("__lichtung_grid_y_0", "Top"),
        ("__lichtung_grid_y_1", "Bottom"),
    ]));
    assert_eq!(render(plot.into(), "grid"), render(native, "grid-native"));
}

#[test]
fn foreign_status_keeps_null_estimates_pending_until_explicit_native_availability() {
    let frame = df!("term" => ["known", "absent"], "estimate" => [Some(2.), None], "status" => [true, false], "state" => ["ok", "unavailable"]).unwrap();
    let plot = crate::Coefficient::from_data(&frame, "term", "estimate")
        .unwrap()
        .estimate_status_categorical(
            "state",
            [
                ("ok", crate::EstimateStatus::Estimated),
                ("unavailable", crate::EstimateStatus::NotEstimable),
            ],
        )
        .unwrap();
    let boolean = crate::Coefficient::from_data(&frame, "term", "estimate")
        .unwrap()
        .estimate_status_boolean("status")
        .unwrap();
    assert_eq!(
        render(plot.width(600).into(), "status"),
        render(boolean.width(600).into(), "status-boolean")
    );
    assert!(crate::Coefficient::from_data(&frame, "term", "estimate")
        .unwrap()
        .estimate_status_categorical("state", [("ok", crate::EstimateStatus::Estimated)])
        .is_err());
}
