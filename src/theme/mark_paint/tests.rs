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

#[test]
fn mapped_and_fixed_outlines_keep_fill_and_stroke_alpha_independent() {
    let outline = OutlineTheme::new(Color::rgba(41, 59, 73, 97), 1.875).unwrap();
    let fixed = DataOutlineTheme::fixed(outline.clone());
    assert_eq!(fixed.fixed_stroke(), Some(&outline));
    assert!(fixed.mapped_width().is_none());
    assert!(fixed.mapped_alpha().is_none());
    let mapped = DataOutlineTheme::mapped(2.375, 211).unwrap();
    assert!(mapped.fixed_stroke().is_none());
    assert_eq!(
        mapped.mapped_width().unwrap().to_bits(),
        2.375_f32.to_bits()
    );
    assert_eq!(mapped.mapped_alpha(), Some(211));
    let source = DataFillTheme::new(17, Some(mapped.clone())).unwrap();
    assert_eq!(source.fill_alpha(), 17);
    assert_eq!(source.outline(), Some(&mapped));
    let style = source.backend_style((13, 29, 47));
    assert_eq!(style.color, Color::rgba(13, 29, 47, 17));
    assert_eq!(style.stroke, Color::rgba(13, 29, 47, 211));
    assert_eq!(style.stroke_width.to_bits(), 2.375_f32.to_bits());
    let changed = source.clone().with_outline(Some(fixed.clone())).unwrap();
    assert_eq!(changed.fill_alpha(), 17);
    assert_eq!(changed.outline(), Some(&fixed));
    let style = changed.backend_style((13, 29, 47));
    assert_eq!(style.color, Color::rgba(13, 29, 47, 17));
    assert_eq!(style.stroke, Color::rgba(41, 59, 73, 97));
    assert_eq!(style.stroke_width.to_bits(), 1.875_f32.to_bits());
    for alpha in [0, 255] {
        let edited = source.clone().with_fill_alpha(alpha).unwrap();
        assert_eq!(edited.fill_alpha(), alpha);
        assert_eq!(edited.outline(), Some(&mapped));
        assert_eq!(edited.backend_style((13, 29, 47)).stroke.alpha(), 211);
        let mapped_alpha =
            DataFillTheme::new(17, Some(DataOutlineTheme::mapped(0.875, alpha).unwrap()))
                .unwrap()
                .backend_style((13, 29, 47));
        assert_eq!(mapped_alpha.color.alpha(), 17);
        assert_eq!(mapped_alpha.stroke.alpha(), alpha);
    }
    let absent = changed.with_outline(None).unwrap();
    assert!(absent.outline().is_none());
    let style = absent.backend_style((13, 29, 47));
    assert_eq!(style.stroke, Color::rgba(0, 0, 0, 0));
    assert_eq!(style.stroke_width.to_bits(), 0.0_f32.to_bits());
    assert_eq!(source.outline(), Some(&mapped));
}

#[test]
fn point_roles_share_the_exact_ratio_and_apply_outline_policy_once() {
    let paint = Color::rgba(11, 23, 37, 53);
    let ratio = RelativeSpacing::ratio(5, 14).unwrap();
    let outline = PointOutlineTheme::new(paint, ratio.clone()).unwrap();
    assert_eq!(outline.color(), paint);
    assert_eq!(outline.width_ratio(), &ratio);
    let other_paint = Color::rgba(41, 59, 73, 0);
    let recolored = outline.clone().with_color(other_paint).unwrap();
    assert_eq!(recolored.color(), other_paint);
    assert_eq!(recolored.width_ratio(), &ratio);
    let other_ratio = RelativeSpacing::ratio(7, 11).unwrap();
    let resized = outline
        .clone()
        .with_width_ratio(other_ratio.clone())
        .unwrap();
    assert_eq!(resized.color(), paint);
    assert_eq!(resized.width_ratio(), &other_ratio);
    assert!(PointOutlineTheme::new(paint, RelativeSpacing::ratio(0, 1).unwrap()).is_err());
    assert!(outline
        .clone()
        .with_width_ratio(RelativeSpacing::new(-0.0).unwrap())
        .is_err());
    let source = PointTheme::new(17, PointShape::TriangleUp, Some(outline.clone())).unwrap();
    assert_eq!(source.fill_alpha(), 17);
    assert_eq!(source.shape(), PointShape::TriangleUp);
    assert_eq!(source.outline(), Some(&outline));

    let style = source.backend_style((83, 97, 109), 14, true);
    assert_eq!(style.color, Color::rgba(83, 97, 109, 17));
    assert_eq!(style.radius, 14);
    assert_eq!(style.shape, PointShape::TriangleUp);
    assert_eq!(style.outline.as_ref().unwrap().color, paint);
    assert_eq!(&style.outline.as_ref().unwrap().width_ratio, &ratio);
    assert_eq!(
        style.outline.unwrap().width_for_radius(14).unwrap().units(),
        5120
    );
    assert!(source
        .backend_style((83, 97, 109), 14, false)
        .outline
        .is_none());
    for alpha in [0, 255] {
        let edited = source.clone().with_fill_alpha(alpha).unwrap();
        assert_eq!(edited.outline(), Some(&outline));
        assert_eq!(edited.shape(), PointShape::TriangleUp);
        assert_eq!(
            edited.backend_style((83, 97, 109), 14, true).color.alpha(),
            alpha
        );
    }
    let changed_shape = source.clone().with_shape(PointShape::Circle).unwrap();
    assert_eq!(changed_shape.fill_alpha(), 17);
    assert_eq!(changed_shape.outline(), Some(&outline));
    assert_eq!(changed_shape.shape(), PointShape::Circle);
    let absent = source
        .clone()
        .with_outline(Some(recolored))
        .unwrap()
        .with_outline(None)
        .unwrap();
    assert_eq!(absent.fill_alpha(), 17);
    assert_eq!(absent.shape(), PointShape::TriangleUp);
    assert!(absent.outline().is_none());
    assert!(absent
        .backend_style((83, 97, 109), 14, true)
        .outline
        .is_none());
    let tiny =
        PointOutlineTheme::new(paint, RelativeSpacing::new(f64::from_bits(1)).unwrap()).unwrap();
    assert!(tiny.backend_style().width_for_radius(1).is_err());
    let large = PointOutlineTheme::new(paint, RelativeSpacing::new(f64::MAX).unwrap()).unwrap();
    assert!(large.backend_style().width_for_radius(1).is_err());
}

#[test]
fn absent_fill_outlines_have_one_operational_identity_and_invalid_widths_still_refuse() {
    let paint = Color::rgba(11, 23, 37, 53);
    let first = FillStyle::new(paint, Color::rgb(41, 59, 73), 0.0);
    let second = FillStyle::new(paint, Color::rgba(83, 97, 109, 127), -0.0);
    assert_eq!(first.color, second.color);
    assert_eq!(first.stroke, Color::rgba(0, 0, 0, 0));
    assert_eq!(first.stroke, second.stroke);
    assert_eq!(first.stroke_width.to_bits(), second.stroke_width.to_bits());
    let canonical = crate::render::primitives::CanonicalFillStyle::new(&first).unwrap();
    assert_eq!(canonical.color, paint);
    assert_eq!(canonical.stroke, Color::rgba(0, 0, 0, 0));
    assert_eq!(canonical.stroke_width.units(), 0);
    for width in [f32::NAN, f32::INFINITY, -1.0] {
        let invalid = FillStyle::new(Color::rgba(0, 0, 0, 0), Color::rgba(0, 0, 0, 0), width);
        assert!(crate::render::primitives::CanonicalFillStyle::new(&invalid).is_err());
    }
}
