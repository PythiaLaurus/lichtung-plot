use ndarray::{array, s, Array2};

use crate::{
    kind, AdapterError, AlphaValues, CategoryValues, Histogram, HistogramBins, HistogramComposite,
    HistogramInput, HistogramKde, HistogramStat, NumericValues, Plot, Raincloud,
};

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
fn histogram_geometry_keeps_independent_strides_sources_and_owned_snapshots() {
    let mut bins = array![[0.5, 1., 0.], [99., 99., 99.], [0.5, 2., 1.]];
    let present = Array2::from_elem((3, 2), true);
    let kde = array![[0., 0.1], [1., 0.2], [2., 0.1]];
    let rugs = array![[1., 0., 0.2]];
    let adapted = Histogram::geometry_from_data(
        (bins.slice(s![..;2, ..;-1]), present.t()),
        Some(&kde.view()),
        Some(&rugs.view()),
        0,
        1,
        2,
        0,
        1,
        0,
        1,
        2,
        "value",
        "density",
    )
    .unwrap();
    // Neither supplemental source is retained, even while the main source is live.
    drop(kde);
    drop(rugs);
    let adapted = adapted
        .fill_alpha_bins(
            0,
            1,
            [] as [(&str, usize); 0],
            AlphaValues::new("alpha", [120, 200]).unwrap(),
        )
        .unwrap()
        .line_alpha_single(180u8)
        .unwrap();
    assert!(matches!(
        adapted.clone().line_alpha_linear(3, (0., 1.), (0, 255)),
        Err(AdapterError::Column {
            index: 3,
            columns: 3,
            ..
        })
    ));
    let native: Plot<kind::Histogram> = adapted.into();
    bins[(0, 2)] = 99.;
    drop(bins);
    drop(present);

    let direct: Histogram<'static> = Histogram::geometry(
        array![0., 1.].view(),
        array![1., 2.].view(),
        array![0.5, 0.5].view(),
        array![0., 1., 2.].view(),
        array![0.1, 0.2, 0.1].view(),
        array![1.].view(),
        array![0.].view(),
        array![0.2].view(),
        "value",
        "density",
    )
    .unwrap();
    let expected = direct
        .fill_alpha_bins(
            array![0., 1.].view(),
            array![1., 2.].view(),
            [] as [(&str, usize); 0],
            AlphaValues::new("alpha", [120, 200]).unwrap(),
        )
        .unwrap()
        .line_alpha_single(180u8)
        .unwrap();
    assert_eq!(
        render(&native, "selected"),
        render(&expected.into(), "direct")
    );
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

#[test]
fn geometry_binding_checks_local_shapes_without_hiding_native_scientific_errors() {
    let bins = array![[0., 1., 0.5], [1., 2., 0.5]];
    let curve = array![0., 2.];
    let rug = array![1.];
    assert!(matches!(
        Histogram::geometry_from_data(
            bins.view(),
            None,
            None,
            0,
            1,
            2,
            0,
            curve.view(),
            rug.view(),
            array![0.].view(),
            array![0.2].view(),
            "value",
            "density"
        ),
        Err(AdapterError::MissingSource { role: "kde_x", .. })
    ));
    assert!(matches!(
        Histogram::geometry(bins.column(0), bins.column(1), bins.column(2), curve.view(),
            rug.view(), rug.view(), array![0.].view(), array![0.2].view(), "value", "density"),
        Err(AdapterError::Binding { role: "kde_y", detail }) if detail.contains("expected 2 slots, got 1")
    ));
    let present = array![true, false];
    let invalid = Histogram::geometry(
        bins.column(0),
        bins.column(1),
        (bins.column(2), present.view()),
        curve.view(),
        array![0.1, 0.1].view(),
        rug.view(),
        array![0.].view(),
        array![0.2].view(),
        "value",
        "density",
    )
    .unwrap();
    let invalid: Plot<kind::Histogram> = invalid.into();
    assert!(matches!(invalid.preflight(), Err(crate::Error::Data(_))));

    let no_hue: Raincloud<'static> = Raincloud::geometry(
        CategoryValues::new("source category", ["a"]).unwrap(),
        array![-1.].view(),
        array![0.].view(),
        array![1.].view(),
        array!["a", "a"].view(),
        array![-2., 2.].view(),
        array![0.5, 0.5].view(),
        array!["a"].view(),
        rug.view(),
        array![0.5].view(),
        "category",
        "value",
    )
    .unwrap();
    let no_hue: Plot<kind::Raincloud> = no_hue.into();
    no_hue.preflight().unwrap();
}

#[test]
fn sample_kde_is_independent_and_does_not_replace_samples_stat_or_bins() {
    let samples = array![[0.], [0.5], [1.5], [2.]];
    let curve = array![[0., 0.1], [1., 0.2], [2., 0.1]];
    let adapted = Histogram::from_data(samples.view(), 0)
        .unwrap()
        .bins(HistogramBins::Count(2));
    assert!(matches!(
        adapted.clone().composite_precomputed_kde(0, 0),
        Err(AdapterError::MissingSource { .. })
    ));
    let adapted = adapted
        .composite_precomputed_kde_from_data(curve.view(), 0, 1)
        .unwrap();
    drop(curve);
    let count_plot: Plot<kind::Histogram> = adapted.clone().into();
    assert!(matches!(count_plot.preflight(), Err(crate::Error::Data(_))));
    let adapted: Plot<kind::Histogram> = adapted.stat(HistogramStat::Density).into();
    drop(samples);
    let expected = Plot::histogram(HistogramInput::samples(NumericValues::new(
        "Column 1",
        [0., 0.5, 1.5, 2.],
    )))
    .bins(HistogramBins::Count(2))
    .stat(HistogramStat::Density)
    .composite(HistogramComposite::KdeAndRug(HistogramKde::Precomputed {
        x: NumericValues::new("x", [0., 1., 2.]),
        y: NumericValues::new("y", [0.1, 0.2, 0.1]),
    }));
    assert_eq!(
        render(&adapted, "sample-kde"),
        render(&expected, "sample-native")
    );
}
