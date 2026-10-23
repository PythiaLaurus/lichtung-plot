use super::*;
use crate::layout::kernel::SpatialRounding;

fn source() -> LegendTheme {
    LegendTheme::builtin().unwrap()
}

#[test]
fn pinned_binary_sources_and_complete_edge_rounding_are_exact() {
    let source = source();

    let lengths = source.lengths(11, 12).unwrap();
    let edge = lengths.padding.add_length(&lengths.glyph_edge_padding);
    assert_eq!(
        edge.round(SpatialRounding::NearestEven).unwrap().quanta(),
        11264
    );
    assert_eq!(edge.round(SpatialRounding::Ceil).unwrap().quanta(), 11265);
    assert_eq!(
        edge.times(2).round(SpatialRounding::Ceil).unwrap().quanta(),
        22529
    );
    assert_ne!(
        source.get_spacing().get_padding(),
        &RelativeSpacing::ratio(4, 5).unwrap()
    );
    assert_ne!(
        source.get_spacing().get_glyph_edge_padding(),
        &RelativeSpacing::ratio(1, 5).unwrap()
    );
    let exact = GradientLengths::new(
        RelativeSpacing::ratio(6, 10).unwrap(),
        RelativeSpacing::ratio(3, 5).unwrap(),
    )
    .unwrap();
    assert_eq!(exact.get_minimum(), exact.get_preferred());
}
