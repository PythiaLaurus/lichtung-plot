use super::*;

#[test]
fn waterfall_rejects_terminal_geometry_below_coordinate_precision() {
    let plot = Plot::waterfall(
        CategoryValues::new("label", ["a", "b"]).unwrap(),
        NumericValues::new("delta", [-10_000_000_000_000_000.0, 1.0]),
        10_000_000_000_000_000.0,
    );

    let error = plot.preflight().unwrap_err();

    assert!(matches!(error, lichtung_plot::Error::Render(_)), "{error}");
    assert!(error.to_string().contains("polygon"), "{error}");
    assert!(error.to_string().contains("collapse"), "{error}");
}

#[test]
fn waterfall_rejects_nonfinite_intermediate_endpoint_with_source_row() {
    let plot = Plot::waterfall(
        CategoryValues::new("label", ["a", "b"]).unwrap(),
        NumericValues::new("delta", [f64::MAX, f64::MAX]),
        0.0,
    );

    assert_eq!(
        plot.preflight().unwrap_err().to_string(),
        "data error: waterfall: cumulative endpoint is non-finite at source row 1"
    );
}
