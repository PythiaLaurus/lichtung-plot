use crate::theme::{LocalMetric, RelativeSpacing};

#[test]
fn publication_title_keeps_the_exact_binary64_leading_and_upward_occupancy() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let title = theme.typography.get_root().get_title();
    assert_eq!(title.get_line_height().to_bits(), 0x3ff4_cccc_cccc_cccd);
    let exact = RelativeSpacing::new(title.get_line_height())
        .unwrap()
        .product(LocalMetric::css_pixels(title.get_font_size()).unwrap());
    let occupancy = exact
        .round(crate::layout::kernel::SpatialRounding::Ceil)
        .unwrap();
    assert_eq!(occupancy.quanta(), 26_625);
}
