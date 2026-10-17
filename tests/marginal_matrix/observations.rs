use lichtung_plot::{NumericValues, Plot, Theme};

#[test]
fn matrix_preserves_observations_and_variable_labels_for_each_theme() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("plot.svg");
    for (name, theme) in [
        ("publication", Theme::publication()),
        ("nature", Theme::nature()),
        ("science", Theme::science()),
        ("chi", Theme::chi()),
    ] {
        Plot::matrix(vec![
            NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
            NumericValues::new("y", [4.0, 3.0, 2.0, 1.0]),
        ])
        .theme(theme)
        .size(800, 600)
        .svg_preserve_text(&path)
        .unwrap();
        let svg = std::fs::read_to_string(&path).unwrap();
        // Four source observations occur in each of the two off-diagonal cells.
        assert_eq!(svg.matches("<circle ").count(), 8, "{name}");
        assert!(
            svg.contains(">X</text>") && svg.contains(">Y</text>"),
            "{name}"
        );
    }
}
