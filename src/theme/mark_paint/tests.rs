use super::*;

#[test]
fn stroke_outline_and_solid_edits_retain_independent_complete_operands() {
    let paint = Color::rgba(13, 29, 47, 61);
    let replacement = Color::rgba(71, 83, 97, 0);
    let stroke = StrokeTheme::new(paint, 1.375, StrokePattern::ShortDash).unwrap();
    assert_eq!(stroke.color(), paint);
    assert_eq!(stroke.width().to_bits(), 1.375_f32.to_bits());
    assert_eq!(stroke.pattern(), StrokePattern::ShortDash);
    assert_eq!(
        stroke.clone().with_color(replacement).unwrap(),
        StrokeTheme {
            paint: replacement,
            ..stroke.clone()
        }
    );
    assert_eq!(
        stroke.clone().with_width(2.625).unwrap(),
        StrokeTheme {
            width: 2.625,
            ..stroke.clone()
        }
    );
    assert_eq!(
        stroke
            .clone()
            .with_pattern(StrokePattern::LongDash)
            .unwrap(),
        StrokeTheme {
            pattern: StrokePattern::LongDash,
            ..stroke.clone()
        }
    );
    let repeated = stroke
        .clone()
        .with_width(3.0)
        .unwrap()
        .with_width(0.75)
        .unwrap();
    assert_eq!(repeated.color(), paint);
    assert_eq!(repeated.width().to_bits(), 0.75_f32.to_bits());
    assert_eq!(repeated.pattern(), StrokePattern::ShortDash);
    for (pattern, expected) in [
        (StrokePattern::Solid, None),
        (StrokePattern::ShortDash, Some("3 2")),
        (StrokePattern::MediumDash, Some("4 3")),
        (StrokePattern::LongDash, Some("5 3")),
    ] {
        let style = stroke
            .clone()
            .with_pattern(pattern)
            .unwrap()
            .backend_style();
        assert_eq!(style.color, paint);
        assert_eq!(style.width.to_bits(), 1.375_f32.to_bits());
        assert_eq!(style.pattern.svg_dasharray(), expected);
    }
    macro_rules! exercise_solid {
        ($type:ident) => {
            let source = $type::new(paint, 1.875).unwrap();
            assert_eq!(source.color(), paint);
            assert_eq!(source.width().to_bits(), 1.875_f32.to_bits());
            assert_eq!(
                source.clone().with_color(replacement).unwrap(),
                $type {
                    paint: replacement,
                    ..source.clone()
                }
            );
            assert_eq!(
                source.clone().with_width(2.375).unwrap(),
                $type {
                    width: 2.375,
                    ..source.clone()
                }
            );
            assert_eq!(source.color(), paint);
            assert_eq!(source.width().to_bits(), 1.875_f32.to_bits());
        };
    }
    exercise_solid!(SolidStrokeTheme);
    exercise_solid!(OutlineTheme);
    let fill = OutlineTheme::new(paint, 1.875)
        .unwrap()
        .backend_style(replacement);
    assert_eq!(fill.color, replacement);
    assert_eq!(fill.stroke, paint);
    assert_eq!(fill.stroke_width.to_bits(), 1.875_f32.to_bits());
}

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
fn fixed_fill_edits_and_absence_retain_only_their_selected_paints() {
    let paint = Color::rgba(11, 23, 37, 53);
    let outline = OutlineTheme::new(Color::rgba(41, 59, 73, 89), 1.625).unwrap();
    let source = FixedFillTheme::new(paint, Some(outline.clone())).unwrap();
    assert_eq!(source.color(), paint);
    assert_eq!(source.outline(), Some(&outline));
    let edited = source
        .clone()
        .with_color(Color::rgba(97, 109, 127, 0))
        .unwrap();
    assert_eq!(edited.outline(), Some(&outline));
    assert_eq!(edited.backend_style().color, Color::rgba(97, 109, 127, 0));
    assert_eq!(edited.backend_style().stroke, outline.color());
    let other = OutlineTheme::new(Color::rgb(131, 149, 163), 2.125).unwrap();
    let changed = source.clone().with_outline(Some(other.clone())).unwrap();
    assert_eq!(changed.color(), paint);
    assert_eq!(changed.outline(), Some(&other));
    let absent = changed.with_outline(None).unwrap();
    assert!(absent.outline().is_none());
    assert_eq!(absent.backend_style().color, paint);
    assert_eq!(absent.backend_style().stroke, Color::rgba(0, 0, 0, 0));
    assert_eq!(absent.backend_style().stroke_width.to_bits(), 0);
    assert_eq!(source.outline(), Some(&outline));
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
fn data_line_edits_replace_one_input_and_never_requantize_alpha() {
    let source = DataLineTheme::new(1.875, 211, StrokePattern::MediumDash).unwrap();
    assert_eq!(source.width().to_bits(), 1.875_f32.to_bits());
    assert_eq!(source.alpha(), 211);
    assert_eq!(source.pattern(), StrokePattern::MediumDash);
    assert_eq!(
        source.clone().with_width(2.625).unwrap(),
        DataLineTheme {
            width: 2.625,
            ..source.clone()
        }
    );
    assert_eq!(
        source.clone().with_alpha(17).unwrap(),
        DataLineTheme {
            alpha: 17,
            ..source.clone()
        }
    );
    assert_eq!(
        source
            .clone()
            .with_pattern(StrokePattern::LongDash)
            .unwrap(),
        DataLineTheme {
            pattern: StrokePattern::LongDash,
            ..source.clone()
        }
    );
    for alpha in [0, 255] {
        let edited = source.clone().with_alpha(alpha).unwrap();
        let style = edited.backend_style((13, 29, 47));
        assert_eq!(style.color, Color::rgba(13, 29, 47, alpha));
        assert_eq!(style.width.to_bits(), 1.875_f32.to_bits());
        assert_eq!(style.pattern, StrokePattern::MediumDash);
    }
    assert_eq!(source.alpha(), 211);
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

#[test]
fn canonical_absent_outline_metadata_preserves_png_pixels() {
    use crate::render::backend::{Backend, Point2D};
    use crate::render::svg_backend::{rasterize_svg_bytes_to_png, PngTargetPlan, SvgBackend};
    let current = DataFillTheme::new(56, None)
        .unwrap()
        .backend_style((0, 114, 178));
    // Independent raw backend fixture for the prior unpainted metadata. This is
    // not a foundation value or a second production outline representation.
    let prior_metadata = FillStyle {
        color: Color::rgba(0, 114, 178, 56),
        stroke: Color::rgb(0, 114, 178),
        stroke_width: 0.0,
    };
    let svg = |style: &FillStyle| {
        let mut backend = SvgBackend::in_memory(80, 60);
        backend.begin(80, 60).unwrap();
        backend
            .draw_polygon(
                &[
                    Point2D::new(10.0, 10.0),
                    Point2D::new(60.0, 13.0),
                    Point2D::new(55.0, 45.0),
                    Point2D::new(17.0, 40.0),
                ],
                style,
            )
            .unwrap();
        backend.end().unwrap();
        backend.into_bytes().unwrap()
    };
    let current_svg = svg(&current);
    let prior_svg = svg(&prior_metadata);
    assert_ne!(current_svg, prior_svg);
    let options = crate::PngOptions {
        sizing: crate::PngSizing::Source,
        compression: crate::PngCompression::Balanced,
    };
    let size = resvg::tiny_skia::IntSize::from_wh(80, 60).unwrap();
    let plan = PngTargetPlan::new(size, &options).unwrap();
    let current_png = rasterize_svg_bytes_to_png(&current_svg, plan, &options).unwrap();
    let prior_png = rasterize_svg_bytes_to_png(&prior_svg, plan, &options).unwrap();
    assert_eq!(current_png, prior_png);
    let mut reader = png::Decoder::new(std::io::Cursor::new(current_png))
        .read_info()
        .unwrap();
    let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
    let image = reader.next_frame(&mut pixels).unwrap();
    assert_eq!(image.color_type, png::ColorType::Rgba);
    assert_eq!((image.width, image.height), (80, 60));
    assert_eq!(pixels[(30 * 80 + 35) * 4 + 3], 56);
}
