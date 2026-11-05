//! One consumer owns both dependencies; neither adapter depends on the other.
use lichtung_plot_ndarray as nd;
use lichtung_plot_polars as pp;
use ndarray::array;
use polars_core::prelude::*;

fn same_plot<K: pp::kind::Marker>(plot: pp::Plot<K>) -> nd::Plot<K> {
    plot
}

fn native_error_and_stats_paths() {
    let result: pp::Result<()> = Err(nd::Error::Data("native".into()));
    assert!(matches!(result, Err(pp::Error::Data(_))));
    let quantile: fn(&[f64], f64) -> nd::Result<f64> = pp::stats::distribution::quantile;
    assert_eq!(
        quantile(&[1., 3.], 0.5).unwrap().to_bits(),
        2.0_f64.to_bits()
    );
}

#[test]
fn native_paths_and_source_handoffs_preserve_canvas_label_inheritance() {
    native_error_and_stats_paths();
    let frame =
        df!("x_value"=>[0.,5.,20.],"y_value"=>[1.,2.,3.],"condition"=>["one","one","two"]).unwrap();
    let temporary = Series::new("y_value".into(), [1., 2., 3.]);
    let left = pp::Scatter::from_data(&frame, "x_value", &temporary)
        .unwrap()
        .hue("condition")
        .unwrap();
    drop(temporary);
    let left: nd::Plot<nd::kind::Scatter> = same_plot(left.into());
    let shared = pp::Matrix::from_data(&frame, ["y_value", "x_value"]).unwrap();
    let _: nd::Plot<nd::kind::Matrix> = shared.into();
    assert!(matches!(
        pp::Scatter::from_data(&frame, "x_value", "condition"),
        Err(pp::AdapterError::UnsupportedDtype { channel: "y", .. })
    ));
    let missing = pp::Scatter::from_data(&frame, "missing", "y_value")
        .err()
        .unwrap();
    assert!(matches!(
        missing,
        pp::AdapterError::Lookup { role: "x", .. }
    ));
    assert!(matches!(
        pp::Scatter::new("x_value", "y_value"),
        Err(pp::AdapterError::MissingSource { .. })
    ));
    drop(frame);

    let mut table = array![[0., 1.], [5., 2.], [20., 3.]];
    let transient = array![1., 2., 3.];
    let right = nd::Scatter::from_data(table.view(), 0, transient.view()).unwrap();
    drop(transient);
    let right: pp::Plot<pp::kind::Scatter> = right.into();
    let matrix = nd::Matrix::from_data(table.view(), [1, 0]).unwrap();
    let _: pp::Plot<pp::kind::Matrix> = matrix.into();
    assert!(matches!(
        nd::Scatter::from_data(table.view(), 8, 1),
        Err(nd::AdapterError::Column { role: "x", .. })
    ));
    assert!(matches!(
        nd::Scatter::from_data(table.view(), 0, 1).unwrap().hue(1),
        Err(nd::AdapterError::UnsupportedElement {
            role: "hue",
            column: Some(1),
            ..
        })
    ));
    assert!(matches!(
        nd::Scatter::new(0usize, 1usize),
        Err(nd::AdapterError::MissingSource { .. })
    ));
    table[(0, 0)] = 99.;
    drop(table);

    let owned: pp::Scatter<'static> = pp::Scatter::new(
        &Series::new("x".into(), [0., 1.]),
        &Series::new("y".into(), [2., 3.]),
    )
    .unwrap();
    let _: nd::Plot<nd::kind::Scatter> = owned.into();
    let owned: nd::Scatter<'static> =
        nd::Scatter::new(array![0., 1.].view(), array![2., 3.].view()).unwrap();
    let inferred_panel: fn(nd::Scatter<'_>) -> nd::Result<nd::Canvas> =
        |plot| nd::Canvas::grid(1, 1)?.panel(0, 0, plot.into());
    let _: pp::Canvas = inferred_panel(owned).unwrap();
    let native_panel: fn(
        pp::Canvas,
        usize,
        usize,
        pp::Plot<pp::kind::Scatter>,
    ) -> pp::Result<pp::Canvas> = pp::Canvas::panel::<pp::kind::Scatter>;

    // Only the crate prefix differs from an ordinary standalone native consumer.
    let native: nd::Plot<nd::kind::Line> = pp::Plot::line(
        pp::NumericValues::new("x_value", [0., 5., 20.]),
        pp::NumericValues::new("y_value", [1., 2., 3.]),
    )
    .band(nd::Bounds::new(
        nd::NumericValues::new("lower", [0.5, 1.5, 2.5]),
        nd::NumericValues::new("upper", [1.5, 2.5, 3.5]),
    ));
    let theme: nd::Theme = pp::Theme::science();
    let canvas: nd::Canvas = pp::Canvas::grid(1, 3)
        .unwrap()
        .theme(theme)
        .labels(nd::Labels::raw());
    let canvas = native_panel(canvas, 0, 0, left)
        .unwrap()
        .panel(0, 1, right)
        .unwrap()
        .panel(0, 2, native)
        .unwrap();
    let path = std::env::temp_dir().join(format!(
        "lichtung-native-adapters-{}.svg",
        std::process::id()
    ));
    let _: pp::Output = canvas.svg_preserve_text(&path).unwrap();
    let svg = std::fs::read_to_string(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(svg.matches(">x_value</text>").count(), 2);
    assert_eq!(svg.matches(">y_value</text>").count(), 2);
    assert!(svg.contains(">Column 1</text>") && svg.contains(">y</text>"));
}
