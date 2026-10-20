use super::*;

#[test]
fn present_widths_reject_invalid_inputs_without_imposing_a_minimum_or_maximum() {
    let paint = Color::rgba(3, 5, 7, 0);
    for width in [0.0, -0.0, -1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(StrokeTheme::new(paint, width, StrokePattern::Solid).is_err());
        assert!(SolidStrokeTheme::new(paint, width).is_err());
        assert!(OutlineTheme::new(paint, width).is_err());
        assert!(DataLineTheme::new(width, 0, StrokePattern::Solid).is_err());
        assert!(DataOutlineTheme::mapped(width, 255).is_err());
        assert!(StrokeTheme::new(paint, 1.0, StrokePattern::Solid)
            .unwrap()
            .with_width(width)
            .is_err());
        assert!(SolidStrokeTheme::new(paint, 1.0)
            .unwrap()
            .with_width(width)
            .is_err());
        assert!(OutlineTheme::new(paint, 1.0)
            .unwrap()
            .with_width(width)
            .is_err());
        assert!(DataLineTheme::new(1.0, 255, StrokePattern::Solid)
            .unwrap()
            .with_width(width)
            .is_err());
    }
    for width in [f32::from_bits(1), f32::MIN_POSITIVE, f32::MAX] {
        assert_eq!(
            StrokeTheme::new(paint, width, StrokePattern::Solid)
                .unwrap()
                .width()
                .to_bits(),
            width.to_bits()
        );
        assert_eq!(
            SolidStrokeTheme::new(paint, width)
                .unwrap()
                .width()
                .to_bits(),
            width.to_bits()
        );
        assert_eq!(
            OutlineTheme::new(paint, width).unwrap().width().to_bits(),
            width.to_bits()
        );
        assert_eq!(
            DataLineTheme::new(width, 255, StrokePattern::Solid)
                .unwrap()
                .width()
                .to_bits(),
            width.to_bits()
        );
        assert_eq!(
            DataOutlineTheme::mapped(width, 0)
                .unwrap()
                .mapped_width()
                .unwrap()
                .to_bits(),
            width.to_bits()
        );
    }
    let poisoned = OutlineTheme {
        paint,
        width: f32::NAN,
    };
    assert!(poisoned.clone().with_color(Color::rgb(1, 2, 3)).is_err());
    assert!(FixedFillTheme::new(paint, Some(poisoned.clone())).is_err());
    assert!(DataFillTheme::new(17, Some(DataOutlineTheme::fixed(poisoned))).is_err());
    assert!(StrokeTheme {
        paint,
        width: 0.0,
        pattern: StrokePattern::Solid
    }
    .with_pattern(StrokePattern::ShortDash)
    .is_err());
    assert!(SolidStrokeTheme { paint, width: 0.0 }
        .with_color(paint)
        .is_err());
    assert!(DataLineTheme {
        width: 0.0,
        alpha: 211,
        pattern: StrokePattern::Solid
    }
    .with_alpha(0)
    .is_err());
    assert!(DataLineTheme {
        width: f32::NAN,
        alpha: 211,
        pattern: StrokePattern::Solid
    }
    .with_pattern(StrokePattern::LongDash)
    .is_err());
}
