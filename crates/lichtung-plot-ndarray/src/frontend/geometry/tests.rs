use ndarray::array;

use crate::{kind, AdapterError, AlphaValues, NumericValues, Plot, Raincloud};

fn render<K: kind::Marker>(plot: &Plot<K>, case: &str) -> String {
    let path = std::env::temp_dir().join(format!(
        "lichtung-ndarray-geometry-{}-{case}.svg",
        std::process::id()
    ));
    plot.svg_preserve_text(&path).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    text
}

#[test]
fn raincloud_geometry_uses_point_rows_for_alpha_but_only_summary_for_lookup() {
    let summary = array![["a", "one"], ["b", "one"]];
    let density = array![["b", "one"], ["a", "one"], ["b", "one"], ["a", "one"]];
    let points = array![[0., 0.25], [0.5, 0.5], [1., 0.75]];
    let adapted = Raincloud::geometry_with_hue_from_data(
        summary.view(),
        Some(&density.view()),
        Some(&points.view()),
        0,
        1,
        array![-1., -1.].view(),
        array![0., 0.].view(),
        array![1., 1.].view(),
        0,
        1,
        array![-2., -2., 2., 2.].view(),
        array![0.5, 0.5, 0.5, 0.5].view(),
        array!["b", "a", "b"].view(),
        array!["one", "one", "one"].view(),
        0,
        1,
        "category",
        "value",
        "cohort",
    )
    .unwrap();
    drop(density);
    drop(points);
    let alpha = array![0., 230., 128.];
    let adapted = adapted
        .point_alpha_linear(alpha.view(), (0., 255.), (0, 255))
        .unwrap();
    assert!(matches!(
        adapted.clone().point_alpha_linear(array![1., 2.].view(), (0., 255.), (0, 255)),
        Err(AdapterError::Binding { role: "alpha", detail }) if detail.contains("expected 3 slots, got 2")
    ));
    // The primary string table must not silently become the numeric points source.
    assert!(matches!(
        adapted.clone().point_alpha_linear(0, (0., 255.), (0, 255)),
        Err(AdapterError::UnsupportedElement { .. })
    ));
    let adapted = adapted
        .line_alpha_keyed_map(
            [("category", 0), ("cohort", 1)],
            NumericValues::new("quality", [0.25, 0.75]),
            |values| AlphaValues::linear(values, (0., 1.), (80, 240)),
        )
        .unwrap()
        .fill_alpha_single(180u8)
        .unwrap();
    let native: Plot<kind::Raincloud> = adapted.labels(crate::Labels::raw()).into();
    drop(summary);
    native.preflight().unwrap();
    let svg = render(&native, "interleaved");
    assert!(svg.contains("category") && svg.contains("value") && svg.contains("cohort"));
}
