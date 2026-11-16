use polars_core::prelude::*;

use crate::{kind, AdapterError, AlphaValues, Plot, Raincloud};

fn render<K: kind::Marker>(plot: &Plot<K>, case: &str) -> String {
    let path = std::env::temp_dir().join(format!(
        "lichtung-polars-geometry-{}-{case}.svg",
        std::process::id()
    ));
    plot.svg_preserve_text(&path).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    text
}

fn enum_column(name: &str, values: &[&str], levels: &[&str]) -> Column {
    Series::new(name.into(), values)
        .strict_cast(&DataType::from_frozen_categories(
            FrozenCategories::new(levels.iter().copied()).unwrap(),
        ))
        .unwrap()
        .into_column()
}

#[test]
fn raincloud_geometry_decodes_independent_keys_and_keeps_original_point_alpha_scope() {
    let summary = DataFrame::new_infer_height(vec![
        enum_column("group", &["a", "b"], &["a", "unused", "b"]),
        Column::new("hue".into(), ["one"; 2]),
        Column::new("q1".into(), [-1.; 2]),
        Column::new("median".into(), [0.; 2]),
        Column::new("q3".into(), [1.; 2]),
    ])
    .unwrap();
    let density = DataFrame::new_infer_height(vec![
        enum_column("group", &["b", "a", "b", "a"], &["b", "a"]),
        Column::new("hue".into(), ["one"; 4]),
        Column::new("y".into(), [-2., -2., 2., 2.]),
        Column::new("width".into(), [0.5; 4]),
    ])
    .unwrap();
    let points = DataFrame::new_infer_height(vec![
        enum_column("group", &["b", "a", "b"], &["unused", "b", "a"]),
        Column::new("hue".into(), ["one"; 3]),
        Column::new("y".into(), [0., 0.5, 1.]),
        Column::new("strip".into(), [0.25, 0.5, 0.75]),
    ])
    .unwrap();
    let adapted = Raincloud::geometry_with_hue_from_data(
        &summary,
        Some(&density),
        Some(&points),
        "group",
        "hue",
        "q1",
        "median",
        "q3",
        "group",
        "hue",
        "y",
        "width",
        "group",
        "hue",
        "y",
        "strip",
        "category",
        "value",
        "cohort",
    )
    .unwrap();
    drop(density);
    drop(points);
    let adapted = adapted
        .point_alpha_linear(
            &Series::new("confidence".into(), [0., 230., 128.]),
            (0., 255.),
            (0, 255),
        )
        .unwrap();
    assert!(matches!(
        adapted
            .clone()
            .point_alpha_linear("q1", (0., 255.), (0, 255)),
        Err(AdapterError::Length {
            expected: 3,
            actual: 2,
            ..
        })
    ));
    assert!(matches!(
        adapted
            .clone()
            .point_alpha_linear("y", (0., 255.), (0, 255)),
        Err(AdapterError::Lookup { .. })
    ));
    let adapted = adapted
        .line_alpha_keyed_map(
            [("category", "group"), ("cohort", "hue")],
            &Series::new("quality".into(), [0.25, 0.75]),
            |values| AlphaValues::linear(values, (0., 1.), (80, 240)),
        )
        .unwrap()
        .fill_alpha_single(180u8)
        .unwrap();
    let native: Plot<kind::Raincloud> = adapted.labels(crate::Labels::raw()).into();
    drop(summary);
    native.preflight().unwrap();
    let svg = render(&native, "independent-keys");
    assert!(svg.contains("category") && svg.contains("value") && svg.contains("cohort"));
}
