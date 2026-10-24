use super::*;
use crate::decorations::legend::descriptor::MissingKey;
use crate::decorations::legend::discrete::PreparedGuide;
use crate::decorations::legend::{Legend, LegendDrawPlan};
use crate::layout::fit::FitOutcome;
use std::sync::Arc;

fn descriptor() -> ContinuousDescriptor {
    ContinuousDescriptor {
        selection: crate::theme::ContinuousColorScale::diverging(
            0.0,
            Some(
                crate::theme::scale_values::test_defaults()
                    .get_diverging()
                    .clone(),
            ),
            None,
        )
        .unwrap(),
        field: "signal".into(),
        title: Arc::new("Signal".into()),
        stops: vec![
            GradientStop::new(0.0, (49, 130, 189)),
            GradientStop::new(0.4, (247, 247, 247)),
            GradientStop::new(1.0, (222, 45, 38)),
        ],
        center: Some(0.0),
        missing: Some(MissingKey {
            fill: crate::Color::rgb(224, 224, 224),
            outline: Some(
                crate::theme::OutlineTheme::new(crate::Color::rgb(120, 120, 120), 1.0).unwrap(),
            ),
            label: "Missing".into(),
        }),
        domain: (-2.0, 3.0),
        exact: false,
        format: None,
        numeric: super::super::descriptor::domain_presentation((-2.0, 3.0), Some(0.0), None)
            .unwrap(),
    }
}

fn prepared(descriptor: &ContinuousDescriptor, theme: &ResolvedTheme) -> Arc<PreparedGuide> {
    let plot = crate::Plot::scatter(
        crate::NumericValues::new("x", [0.0]),
        crate::NumericValues::new("y", [0.0]),
    )
    .erase();
    PreparedGuide::prepare(
        &plot,
        theme,
        &crate::layout::ComponentAddress::root(),
        Vec::new(),
        Some(Arc::new(descriptor.clone())),
        Vec::new(),
        crate::LegendComposition::Combined,
    )
    .unwrap()
    .unwrap()
}

fn measure(
    descriptor: &ContinuousDescriptor,
    available: (f64, f64),
    horizontal: bool,
    compact: bool,
    theme: &ResolvedTheme,
    changes: &mut bool,
) -> FitOutcome<Legend> {
    Legend::measure_assigned(
        prepared(descriptor, theme),
        available,
        horizontal,
        compact,
        theme,
        changes,
        &FitScope::root(),
    )
    .unwrap()
}

fn body(guide: &Legend) -> &MeasuredColorBody {
    guide.layout.sections[0].color.as_ref().unwrap()
}

fn selected(guide: &Legend) -> LegendDrawPlan {
    let (x, y) = guide.box_at(20.0, 30.0).anchor();
    let selected = guide.select(x, y).unwrap();
    let mut ink = crate::render::geometry::GeometrySnapshot::new();
    selected.geometry(&mut ink).unwrap();
    assert!(ink.bounds().iter().all(|bounds| bounds.left >= 20.0
        && bounds.bottom >= 30.0
        && bounds.right <= 20.0 + guide.compute_box_width()
        && bounds.top <= 30.0 + guide.compute_box_height()));
    selected
}

#[test]
fn authored_title_owns_line_alignment_gap_and_complete_measurement_identity() {
    let mut theme = crate::Theme::publication().resolve().unwrap();
    theme.legend = theme
        .legend
        .clone()
        .spacing(
            theme
                .legend
                .get_spacing()
                .clone()
                .title_gap(crate::RelativeSpacing::ratio(1, 1).unwrap())
                .unwrap(),
        )
        .unwrap();
    let mut descriptor = descriptor();
    let title = crate::Text::new("Measured signal\nshort").style(
        theme
            .typography
            .guides
            .legend_title
            .clone()
            .font_size(25)
            .unwrap(),
    );
    descriptor.title = Arc::new(title.clone().alignment(crate::TextAlignment::End));
    let measure = |descriptor: &ContinuousDescriptor| {
        fit::measured(Ok(measure(
            descriptor,
            (350.0, 400.0),
            false,
            false,
            &theme,
            &mut false,
        )))
        .unwrap()
    };
    let aligned = measure(&descriptor);
    let same = measure(&descriptor);
    assert!(Arc::ptr_eq(&aligned.layout, &same.layout));
    let section = &aligned.layout.sections[0];
    let plan = section.title.plan();
    assert_eq!(plan.style.font_size, 25);
    assert!(plan.style.exact_override);
    assert_eq!(
        plan.style.alignment,
        crate::layout::PhysicalTextAlignment::End
    );
    assert_eq!(
        plan.lines[0].translation_x_quanta + plan.lines[0].ink_quanta.unwrap().right,
        plan.lines[1].translation_x_quanta + plan.lines[1].ink_quanta.unwrap().right,
    );
    assert_eq!(
        section.body_top - section.top - plan.height_quanta,
        25 * 1024
    );
    selected(&aligned);
    descriptor.title = Arc::new(title.clone());
    let omitted = measure(&descriptor);
    descriptor.title = Arc::new(title.layout(crate::TextLayout::default()));
    let reset = measure(&descriptor);
    assert!(!Arc::ptr_eq(&omitted.layout, &reset.layout));
    assert_eq!(omitted.layout.height, reset.layout.height);
    assert_eq!(
        reset.layout.sections[0].title.plan().style.alignment,
        crate::layout::PhysicalTextAlignment::Start
    );
}

#[test]
fn unusable_guide_misses_without_shaping() {
    let theme = crate::Theme::publication().resolve().unwrap();
    theme.prepare_font_environment().unwrap();
    let descriptor = prepared(&descriptor(), &theme);
    let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
    assert!(matches!(
        Legend::measure_assigned(
            descriptor,
            (0.0, 0.0),
            false,
            false,
            &theme,
            &mut true,
            &FitScope::root()
        )
        .unwrap(),
        FitOutcome::Miss(_)
    ));
    assert_eq!(
        crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get),
        shapes
    );
}

#[test]
fn projected_extents_require_the_exact_center_and_endpoint_clearance() {
    let gap = ExactLength::pixels(5.0).unwrap();
    let minimum = ExactLength::pixels(40.0).unwrap();
    let projected = |lengths: &[f64], fractions: &[f64]| {
        projected_minimum(lengths, fractions, &gap, &minimum)
            .unwrap()
            .pixels_rounded(SpatialRounding::Ceil)
            .unwrap()
            .to_bits()
    };
    assert_eq!(
        projected(&[4.0, 6.0, 10.0], &[0.0, 0.5, 1.0]),
        42.0_f64.to_bits()
    );
    assert_eq!(projected(&[3.0], &[0.5]), 40.0_f64.to_bits());
    assert_eq!(projected(&[30.0], &[0.5]), 70.0_f64.to_bits());
    assert_eq!(
        projected(&[4.0, 6.0, 10.0], &[0.0, 0.1, 1.0]),
        150.0_f64.to_bits()
    );
}

#[test]
fn constant_domain_large_typography_reserves_singleton_length_in_both_flows() {
    let mut theme = crate::Theme::publication().resolve().unwrap();
    theme.typography.guides.legend_label.font_size = 96;
    theme.typography.guides.legend_label.weight = crate::TextWeight::Bold;
    let spacing = theme
        .legend
        .get_spacing()
        .clone()
        .row_gap(crate::RelativeSpacing::from_pixels(5.0, 96).unwrap())
        .unwrap();
    let lengths = theme
        .legend
        .get_gradient_lengths()
        .clone()
        .minimum(crate::RelativeSpacing::from_pixels(40.0, 96).unwrap())
        .unwrap()
        .preferred(crate::RelativeSpacing::from_pixels(512.0, 96).unwrap())
        .unwrap();
    theme.legend = theme
        .legend
        .clone()
        .spacing(spacing)
        .unwrap()
        .gradient_lengths(lengths)
        .unwrap();
    let mut descriptor = descriptor();
    descriptor.domain = (12_345.0, 12_345.0);
    descriptor.center = None;
    descriptor.missing = None;
    descriptor.numeric =
        super::super::descriptor::domain_presentation(descriptor.domain, None, None).unwrap();
    for horizontal in [false, true] {
        let preferred = fit::measured(Ok(measure(
            &descriptor,
            (1200.0, 1200.0),
            horizontal,
            false,
            &theme,
            &mut false,
        )))
        .unwrap();
        let compact = fit::measured(Ok(measure(
            &descriptor,
            (1200.0, 1200.0),
            horizontal,
            true,
            &theme,
            &mut false,
        )))
        .unwrap();
        let length = |guide: &Legend| {
            if horizontal {
                body(guide).gradient.width()
            } else {
                body(guide).gradient.height()
            }
        };
        let minimum = LegendMetrics::from_theme(&theme, 96).unwrap().minimum;
        assert!(length(&compact) > minimum);
        assert!(length(&preferred) > length(&compact));
        for guide in [&preferred, &compact] {
            let body = body(guide);
            assert_eq!(body.labels.len(), 1);
            assert_eq!(body.labels[0].plan.source, descriptor.numeric.labels[0]);
            assert_eq!(body.labels[0].plan.style.font_size, 96);
            assert_eq!(body.stops, descriptor.stops);
            let midpoint = if horizontal {
                f64::midpoint(body.gradient.x0, body.gradient.x1)
            } else {
                f64::midpoint(body.gradient.y0, body.gradient.y1)
            };
            let anchor = if horizontal {
                body.labels[0].placement.position.x
            } else {
                body.labels[0].placement.position.y
            };
            assert!((anchor - midpoint).abs() <= 1.0 / 1024.0);
            selected(guide);
        }
        let minimum = (compact.compute_box_width(), compact.compute_box_height());
        let refused = if horizontal {
            (minimum.0 - 1.0 / 1024.0, minimum.1)
        } else {
            (minimum.0, minimum.1 - 1.0 / 1024.0)
        };
        let repeated = measure(&descriptor, minimum, horizontal, true, &theme, &mut false);
        if let FitOutcome::Miss(failure) = repeated {
            panic!(
                "the measured allocation must admit itself, horizontal={horizontal}: {failure:?}"
            );
        }
        assert!(matches!(
            measure(&descriptor, refused, horizontal, true, &theme, &mut false),
            FitOutcome::Miss(_)
        ));
    }
}

#[test]
fn asymmetric_numeric_centers_keep_their_fraction_or_return_typed_fit() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let mut descriptor = descriptor();
    descriptor.missing = None;
    descriptor.domain = (-0.01, 100.0);
    descriptor.numeric =
        super::super::descriptor::domain_presentation(descriptor.domain, descriptor.center, None)
            .unwrap();
    for horizontal in [false, true] {
        let FitOutcome::Miss(failure) = measure(
            &descriptor,
            (300.0, 300.0),
            horizontal,
            false,
            &theme,
            &mut false,
        ) else {
            panic!("near-endpoint center cannot have overlapping numbers")
        };
        assert_eq!(failure.reason.kind, FitKind::LegendOrGuide);
    }
    descriptor.domain = (-1.0, 9.0);
    descriptor.numeric =
        super::super::descriptor::domain_presentation(descriptor.domain, descriptor.center, None)
            .unwrap();
    for horizontal in [false, true] {
        let guide = fit::measured(Ok(measure(
            &descriptor,
            (600.0, 600.0),
            horizontal,
            true,
            &theme,
            &mut false,
        )))
        .unwrap();
        let body = body(&guide);
        let expected = if horizontal {
            body.gradient.x0 + body.gradient.width() * 0.1
        } else {
            body.gradient.y0 + body.gradient.height() * 0.1
        };
        let position = body.labels[1].placement.position;
        assert_eq!(
            (if horizontal { position.x } else { position.y }).to_bits(),
            expected.to_bits()
        );
        assert!(body
            .labels_have_clearance(horizontal, guide.metrics.row_gap)
            .unwrap());
        assert_eq!(body.stops, descriptor.stops);
    }
}

#[test]
fn horizontal_and_vertical_guides_freeze_one_gradient_and_missing_block() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let descriptor = descriptor();
    for horizontal in [false, true] {
        let guide = fit::measured(Ok(measure(
            &descriptor,
            (300.0, 300.0),
            horizontal,
            false,
            &theme,
            &mut false,
        )))
        .unwrap();
        let body = body(&guide);
        assert_eq!(body.stops, descriptor.stops);
        assert_eq!(body.labels.len(), 3);
        assert!(body.missing.is_some());
        let calls = selected(&guide).calls;
        assert_eq!(
            calls
                .iter()
                .filter(|call| matches!(call, LegendCall::Background(..)))
                .count(),
            1
        );
        assert_eq!(
            calls
                .iter()
                .filter(|call| matches!(call, LegendCall::Gradient(..)))
                .count(),
            1
        );
        assert_eq!(
            calls
                .iter()
                .filter(|call| matches!(call, LegendCall::Fill(..)))
                .count(),
            1
        );
    }
}

#[test]
fn custom_frame_and_owner_missing_strokes_are_allocated_and_retained_unclipped() {
    let mut theme = crate::Theme::publication().resolve().unwrap();
    theme.legend = theme
        .legend
        .clone()
        .frame(Some(
            crate::theme::LegendFrameTheme::new(
                crate::Color::rgba(1, 2, 3, 127),
                Some(
                    crate::theme::OutlineTheme::new(crate::Color::rgba(4, 5, 6, 210), 30.0)
                        .unwrap(),
                ),
                crate::RelativeSpacing::from_pixels(18.0, 11).unwrap(),
            )
            .unwrap(),
        ))
        .unwrap();
    let mut descriptor = descriptor();
    descriptor
        .missing
        .as_mut()
        .unwrap()
        .outline
        .as_mut()
        .unwrap()
        .width = 40.0;
    for horizontal in [false, true] {
        let guide = fit::measured(Ok(measure(
            &descriptor,
            (400.0, 400.0),
            horizontal,
            true,
            &theme,
            &mut false,
        )))
        .unwrap();
        let calls = selected(&guide).calls;
        let LegendCall::Background(rect, fill) = &calls[0] else {
            panic!("one frame is retained first")
        };
        assert_eq!(rect[4].to_bits(), 18.0_f64.to_bits());
        assert_eq!(fill.color, theme.legend.get_frame().unwrap().get_fill());
        assert_eq!(fill.stroke_width.to_bits(), 30.0_f32.to_bits());
        let missing = calls
            .iter()
            .find_map(|call| {
                if let LegendCall::Fill(_, fill) = call {
                    Some(fill)
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(missing.stroke_width.to_bits(), 40.0_f32.to_bits());
        assert!(matches!(
            measure(
                &descriptor,
                (
                    guide.compute_box_width() - 1.0 / 1024.0,
                    guide.compute_box_height()
                ),
                horizontal,
                true,
                &theme,
                &mut false
            ),
            FitOutcome::Miss(_)
        ));
    }
}

#[test]
fn custom_text_ink_overhang_cannot_be_hidden_by_the_guide_frame() {
    let mut theme = crate::Theme::publication().resolve().unwrap();
    theme.typography.guides.legend_title.font_size = 40;
    theme.typography.guides.legend_title.weight = crate::TextWeight::Bold;
    let guide = fit::measured(Ok(measure(
        &descriptor(),
        (300.0, 300.0),
        false,
        false,
        &theme,
        &mut false,
    )))
    .unwrap();
    selected(&guide);
    let descriptor = prepared(&descriptor(), &theme);
    let mut calls = selected(&guide).calls;
    let title = calls
        .iter_mut()
        .find_map(|call| match call {
            LegendCall::Text(text)
                if text.plan.source == descriptor.sections[0].display_title.as_str() =>
            {
                Some(text)
            }
            _ => None,
        })
        .unwrap();
    title.placement.position.x = 20.0 + guide.compute_box_width() - 1.0;
    let mut ink = crate::render::geometry::GeometrySnapshot::new();
    LegendDrawPlan { calls }.geometry(&mut ink).unwrap();
    assert!(ink
        .bounds()
        .iter()
        .any(|bounds| bounds.right > 20.0 + guide.compute_box_width()));
}

#[test]
fn absent_continuous_frame_retains_gradient_labels_and_missing_key() {
    let mut theme = crate::Theme::publication().resolve().unwrap();
    theme.legend = theme.legend.clone().frame(None).unwrap();
    let mut descriptor = descriptor();
    descriptor.missing.as_mut().unwrap().outline = None;
    let guide = fit::measured(Ok(measure(
        &descriptor,
        (500.0, 500.0),
        false,
        false,
        &theme,
        &mut false,
    )))
    .unwrap();
    let calls = selected(&guide).calls;
    assert!(!calls
        .iter()
        .any(|call| matches!(call, LegendCall::Background(..))));
    assert!(calls
        .iter()
        .any(|call| matches!(call, LegendCall::Gradient(..))));
    let labels = calls
        .iter()
        .filter_map(|call| {
            if let LegendCall::Text(text) = call {
                Some(text.plan.source.as_str())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(labels[0], descriptor.title.as_str());
    assert_eq!(labels.last().copied(), Some("Missing"));
    assert_eq!(
        &labels[1..=descriptor.numeric.labels.len()],
        descriptor
            .numeric
            .labels
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
    );
}

#[test]
fn compact_gradient_keeps_all_labels_and_skips_duplicate_allocations() {
    let mut theme = crate::Theme::publication().resolve().unwrap();
    let descriptor = descriptor();
    let mut changes = false;
    let preferred = fit::measured(Ok(measure(
        &descriptor,
        (300.0, 300.0),
        false,
        false,
        &theme,
        &mut changes,
    )))
    .unwrap();
    assert!(changes);
    let compact = fit::measured(Ok(measure(
        &descriptor,
        (300.0, 300.0),
        false,
        true,
        &theme,
        &mut changes,
    )))
    .unwrap();
    assert!(body(&compact).gradient.height() < body(&preferred).gradient.height());
    assert_eq!(body(&compact).labels.len(), body(&preferred).labels.len());
    assert_eq!(body(&compact).stops, body(&preferred).stops);
    let lengths = theme
        .legend
        .get_gradient_lengths()
        .clone()
        .preferred(theme.legend.get_gradient_lengths().get_minimum().clone())
        .unwrap();
    theme.legend = theme.legend.clone().gradient_lengths(lengths).unwrap();
    measure(
        &descriptor,
        (300.0, 300.0),
        false,
        false,
        &theme,
        &mut changes,
    );
    assert!(
        !changes,
        "identical compact geometry must not consume another trial"
    );
}
