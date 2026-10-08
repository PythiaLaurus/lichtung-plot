use super::Plot;

#[test]
fn configuration_clone_preserves_fields_and_shares_scientific_sources() {
    for count in [2, 10_000] {
        let plot = Plot::scatter(
            crate::NumericValues::new("source x", (0..count).map(f64::from)),
            crate::NumericValues::new("source y", (0..count).map(f64::from)),
        )
        .title("Long root title".repeat(80))
        .subtitle("Subtitle")
        .caption("caption")
        .x_label("Explicit x")
        .y_label("Explicit y")
        .labels(crate::Labels::raw().alias("source x", "Display sample"))
        .x_ticks(vec![crate::AxisTick::new(1.0, "one".repeat(120))])
        .erase();
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        let retained = plot.clone();
        assert!(std::sync::Arc::ptr_eq(&plot.input, &retained.input));
        assert!(std::sync::Arc::ptr_eq(
            &plot.histogram_composite,
            &retained.histogram_composite
        ));
        assert_eq!(plot.labels, retained.labels);
        assert_eq!(plot.x_axis_spec, retained.x_axis_spec);
        assert_eq!(plot.title, retained.title);
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }
}
