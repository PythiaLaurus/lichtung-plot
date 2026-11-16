use super::*;

#[test]
fn canvas_region_overwrite_frees_partial_owners_and_renders_only_replacement_anchors() {
    let plot = |title| {
        Plot::scatter(
            NumericValues::new("x", [0., 1.]),
            NumericValues::new("y", [1., 2.]),
        )
        .title(title)
    };
    let canvas = Canvas::grid(2, 3)
        .unwrap()
        .panel_span(0, 0, 2, 2, plot("Removed A"))
        .unwrap()
        .text_panel(
            0,
            2,
            lichtung_plot::TextPanel::new("Removed B", "Body").unwrap(),
        )
        .unwrap()
        .panel(1, 2, plot("Removed C"))
        .unwrap()
        .set_panel(1, 1, plot("Removed X"))
        .unwrap()
        // (0,0) lies outside X's footprint but was freed with the whole A owner.
        .panel(0, 0, plot("Edge neighbor"))
        .unwrap()
        .set_panel_span(0, 1, 2, 2, plot("Joined replacement"))
        .unwrap();
    let svg = render_canvas(canvas, "region-overwrite").unwrap();
    for removed in ["Removed A", "Removed B", "Removed C", "Removed X"] {
        assert!(!svg.contains(removed));
    }
    assert_eq!(svg.matches(">Edge neighbor</text>").count(), 1);
    assert_eq!(svg.matches(">Joined replacement</text>").count(), 1);
    assert_unique_svg_ids(&svg);
}
