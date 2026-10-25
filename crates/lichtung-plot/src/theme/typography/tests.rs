use crate::theme::{LocalMetric, RelativeSpacing};

#[test]
fn fractional_roles_validate_whole_appearance_and_resolve_exact_local_spacing() {
    let base = crate::Theme::publication();
    let role = base
        .typography()
        .get_root()
        .get_title()
        .clone()
        .font_size(10.25)
        .unwrap()
        .line_height(1.5)
        .unwrap();
    let em = LocalMetric::css_pixels(role.get_font_size()).unwrap();
    let pitch = RelativeSpacing::ratio(3, 2).unwrap().product(em);
    assert_eq!(
        pitch
            .round(crate::layout::kernel::SpatialRounding::Ceil)
            .unwrap()
            .quanta(),
        15744
    );
    assert_ne!(role, role.clone().font_size(10.5).unwrap());
    for invalid in [0.0, -0.0, -1.0, f64::INFINITY, f64::NAN] {
        assert!(role.clone().font_size(invalid).is_err());
        assert!(crate::SignificanceMarkStyle::default()
            .font_size(invalid)
            .is_err());
    }
}
