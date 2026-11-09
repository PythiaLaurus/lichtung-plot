use polars_core::prelude::*;

use crate::{
    kind, AdapterError, AlphaValues, Histogram, HistogramBins, HistogramComposite, HistogramInput,
    HistogramKde, HistogramStat, NumericValues, Plot, Raincloud,
};

fn render<K: kind::Marker>(plot: Plot<K>, case: &str) -> String {
    let path = std::env::temp_dir().join(format!(
        "lichtung-polars-geometry-{}-{case}.svg",
        std::process::id()
    ));
    plot.svg_preserve_text(&path).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    text
}

#[test]
fn histogram_geometry_snapshots_chunked_components_and_retains_only_bins_lookup() {
    let mut density = Series::new("height".into(), [0.5]);
    density
        .append(&Series::new("height".into(), [0.5]))
        .unwrap();
    let mut bins =
        df!("lower" => [0., 1.], "upper" => [1., 2.], "unused" => ["meta", "data"]).unwrap();
    bins.with_column(density.into_column()).unwrap();
    let kde = df!("x" => [0., 1., 2.], "y" => [0.1, 0.2, 0.1]).unwrap();
    let rugs = df!("x" => [1.], "base" => [0.], "top" => [0.2]).unwrap();
    let adapted = Histogram::geometry_from_data(
        &bins,
        Some(&kde),
        Some(&rugs),
        "lower",
        "upper",
        "height",
        "x",
        "y",
        "x",
        "base",
        "top",
        "value",
        "density",
    )
    .unwrap();
    drop(kde);
    drop(rugs);
    let adapted = adapted
        .fill_alpha_bins(
            "lower",
            "upper",
            [] as [(&str, &str); 0],
            AlphaValues::new("alpha", [120, 200]).unwrap(),
        )
        .unwrap()
        .line_alpha_single(180u8)
        .unwrap();
    assert!(matches!(
        adapted.clone().line_alpha_linear("x", (0., 1.), (0, 255)),
        Err(AdapterError::Lookup { .. })
    ));
    let native: Plot<kind::Histogram> = adapted.into();
    bins.replace("lower", Column::new("lower".into(), [99., 99.]))
        .unwrap();
    drop(bins);

    let direct: Histogram<'static> = Histogram::geometry(
        &Series::new("lower".into(), [0., 1.]),
        &Series::new("upper".into(), [1., 2.]),
        &Series::new("height".into(), [0.5, 0.5]),
        &Series::new("x".into(), [0., 1., 2.]),
        &Series::new("y".into(), [0.1, 0.2, 0.1]),
        &Series::new("x".into(), [1.]),
        &Series::new("base".into(), [0.]),
        &Series::new("top".into(), [0.2]),
        "value",
        "density",
    )
    .unwrap();
    let expected = direct
        .fill_alpha_bins(
            &Series::new("lower".into(), [0., 1.]),
            &Series::new("upper".into(), [1., 2.]),
            [] as [(&str, &str); 0],
            AlphaValues::new("alpha", [120, 200]).unwrap(),
        )
        .unwrap()
        .line_alpha_single(180u8)
        .unwrap();
    assert_eq!(
        render(native, "selected"),
        render(expected.into(), "direct")
    );
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
    let svg = render(native, "independent-keys");
    assert!(svg.contains("category") && svg.contains("value") && svg.contains("cohort"));
}

#[test]
fn geometry_binding_checks_local_shapes_and_defers_scientific_missingness() {
    let bins =
        df!("lower" => [0., 1.], "upper" => [1., 2.], "height" => [Some(0.5), None]).unwrap();
    let x = Series::new("x".into(), [0., 2.]);
    let y = Series::new("y".into(), [0.1, 0.1]);
    let rug = Series::new("rug".into(), [1.]);
    let base = Series::new("base".into(), [0.]);
    let top = Series::new("top".into(), [0.2]);
    assert!(matches!(
        Histogram::geometry_from_data(
            &bins, None, None, "lower", "upper", "height", "lower", &y, &rug, &base, &top, "value",
            "density",
        ),
        Err(AdapterError::MissingSource { role: "kde_x", .. })
    ));
    assert!(matches!(
        Histogram::geometry(
            bins.column("lower").unwrap(),
            bins.column("upper").unwrap(),
            bins.column("height").unwrap(),
            &x,
            &rug,
            &rug,
            &base,
            &top,
            "value",
            "density",
        ),
        Err(AdapterError::Length {
            role: "kde_y",
            expected: 2,
            actual: 1,
            ..
        })
    ));
    let invalid = Histogram::geometry_from_data(
        &bins, None, None, "lower", "upper", "height", &x, &y, &rug, &base, &top, "value",
        "density",
    )
    .unwrap();
    let invalid: Plot<kind::Histogram> = invalid.into();
    assert!(matches!(invalid.preflight(), Err(crate::Error::Data(_))));
    let category = Series::new("group".into(), ["a"]);
    let no_hue: Raincloud<'static> = Raincloud::geometry(
        &category,
        &Series::new("q1".into(), [-1.]),
        &base,
        &rug,
        &Series::new("group".into(), ["a", "a"]),
        &Series::new("y".into(), [-2., 2.]),
        &y,
        &category,
        &base,
        &Series::new("strip".into(), [0.5]),
        "category",
        "value",
    )
    .unwrap();
    let no_hue: Plot<kind::Raincloud> = no_hue.into();
    no_hue.preflight().unwrap();
}

#[test]
fn sample_kde_source_is_not_retained_and_stat_samples_bins_are_unchanged() {
    let samples = df!("samples" => [0., 0.5, 1.5, 2.]).unwrap();
    let curve = df!("x" => [0., 1., 2.], "y" => [0.1, 0.2, 0.1]).unwrap();
    let adapted = Histogram::from_data(&samples, "samples")
        .unwrap()
        .bins(HistogramBins::Count(2));
    assert!(matches!(
        adapted
            .clone()
            .composite_precomputed_kde("samples", "samples"),
        Err(AdapterError::MissingSource { .. })
    ));
    let adapted = adapted
        .composite_precomputed_kde_from_data(&curve, "x", "y")
        .unwrap();
    drop(curve);
    let count_plot: Plot<kind::Histogram> = adapted.clone().into();
    assert!(matches!(count_plot.preflight(), Err(crate::Error::Data(_))));
    let adapted: Plot<kind::Histogram> = adapted.stat(HistogramStat::Density).into();
    drop(samples);
    let expected = Plot::histogram(HistogramInput::samples(NumericValues::new(
        "samples",
        [0., 0.5, 1.5, 2.],
    )))
    .bins(HistogramBins::Count(2))
    .stat(HistogramStat::Density)
    .composite(HistogramComposite::KdeAndRug(HistogramKde::Precomputed {
        x: NumericValues::new("x", [0., 1., 2.]),
        y: NumericValues::new("y", [0.1, 0.2, 0.1]),
    }));
    assert_eq!(
        render(adapted, "sample-kde"),
        render(expected, "sample-native")
    );
}
