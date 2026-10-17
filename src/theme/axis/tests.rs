use super::*;
use crate::render::backend::StrokePattern;
use crate::theme::Color;

fn stroke(width: f32) -> StrokeTheme {
    StrokeTheme::new(Color::rgba(3, 5, 7, 91), width, StrokePattern::ShortDash).unwrap()
}

fn spacing() -> AxisSpacing {
    AxisSpacing::new(
        RelativeSpacing::ratio(8, 16).unwrap(),
        RelativeSpacing::ratio(6, 16).unwrap(),
        RelativeSpacing::ratio(8, 17).unwrap(),
        RelativeSpacing::ratio(16, 17).unwrap(),
        RelativeSpacing::ratio(16, 17).unwrap(),
        RelativeSpacing::ratio(12, 17).unwrap(),
    )
    .unwrap()
}

fn complete() -> AxisTheme {
    AxisTheme::new(
        Some(
            FrameTheme::new(
                FrameSides::new(true, true, true, true).unwrap(),
                stroke(0.8),
            )
            .unwrap(),
        ),
        Some(
            FrameTheme::new(
                FrameSides::new(false, false, true, true).unwrap(),
                stroke(0.8),
            )
            .unwrap(),
        ),
        Some(TickTheme::new(stroke(1.25), 5).unwrap()),
        Some(stroke(0.5)),
        spacing(),
    )
    .unwrap()
}

#[test]
fn frame_tick_and_axis_edits_revalidate_complete_siblings_and_preserve_absence() {
    let source = complete();
    let frame = source.frame().unwrap();
    let left = FrameSides::new(false, false, false, true).unwrap();
    let edited = frame.clone().with_sides(left.clone()).unwrap();
    assert_eq!(edited.sides(), &left);
    assert_eq!(edited.stroke(), frame.stroke());
    let edited = frame.clone().with_stroke(stroke(3.0)).unwrap();
    assert_eq!(edited.sides(), frame.sides());
    assert_eq!(edited.stroke(), &stroke(3.0));
    let ticks = source.ticks().unwrap();
    assert!(TickTheme::new(stroke(1.0), 0).is_err());
    assert!(ticks.clone().with_length(0).is_err());
    assert_eq!(
        ticks.clone().with_length(u32::MAX).unwrap().length(),
        u32::MAX
    );
    assert_eq!(
        ticks.clone().with_stroke(stroke(2.0)).unwrap().stroke(),
        &stroke(2.0)
    );
    assert_eq!(
        ticks.clone().with_length(9).unwrap().stroke(),
        ticks.stroke()
    );
    let absent = source
        .clone()
        .with_frame(None)
        .unwrap()
        .with_cell_frame(None)
        .unwrap()
        .with_ticks(None)
        .unwrap()
        .with_grid(None)
        .unwrap();
    assert!(
        absent.frame().is_none()
            && absent.cell_frame().is_none()
            && absent.ticks().is_none()
            && absent.grid().is_none()
    );
    assert_eq!(absent.spacing(), source.spacing());
    assert_eq!(source.clone().with_spacing(spacing()).unwrap(), source);
    assert_eq!(
        source.clone().with_frame(Some(edited)).unwrap().ticks(),
        source.ticks()
    );
    for width in [f32::from_bits(1), f32::MAX] {
        assert_eq!(
            FrameTheme::new(left.clone(), stroke(width))
                .unwrap()
                .stroke()
                .width()
                .to_bits(),
            width.to_bits()
        );
        assert_eq!(
            TickTheme::new(stroke(width), u32::MAX).unwrap().length(),
            u32::MAX
        );
        assert!(stroke_style(&stroke(width)).is_err());
    }
    let mut poisoned = source.clone();
    poisoned.frame.as_mut().unwrap().stroke.width = f32::NAN;
    assert!(poisoned.clone().with_ticks(None).is_err());
    assert!(poisoned.clone().with_grid(None).is_err());
    assert!(poisoned.with_spacing(spacing()).is_err());
    let mut poisoned = ticks.clone();
    poisoned.length = 0;
    assert!(poisoned.with_stroke(stroke(3.0)).is_err());
    let mut poisoned = frame.clone();
    poisoned.sides = FrameSides { bits: 0 };
    assert!(poisoned.clone().with_stroke(stroke(3.0)).is_err());
    assert!(AxisTheme::new(Some(poisoned), None, None, None, spacing()).is_err());
}

#[test]
fn complete_hard_sums_keep_fractional_gaps_until_the_single_rounding_boundary() {
    let first = RelativeSpacing::ratio(1, 3072)
        .unwrap()
        .product(LocalMetric::css_pixels(1).unwrap());
    let sum = first
        .add_length(&first)
        .add_length(&first)
        .add_pixels(1.0)
        .unwrap();
    assert_eq!(
        AxisLengths::hard(&sum).unwrap().to_bits(),
        (1025.0_f64 / 1024.0).to_bits()
    );
    assert_eq!(AxisLengths::position(&first).unwrap().to_bits(), 0);
    assert_eq!(
        AxisLengths::hard(&first).unwrap().to_bits(),
        (1.0_f64 / 1024.0).to_bits()
    );
    let huge = ExactLength::pixels(f64::MAX).unwrap();
    assert!(AxisLengths::position(&huge).is_err());
}

#[test]
fn whole_positions_round_half_quantum_ties_with_the_final_anchor_parity() {
    let half = RelativeSpacing::ratio(1, 2048)
        .unwrap()
        .product(LocalMetric::css_pixels(1).unwrap());
    for (anchor, expected) in [
        (0.0, 0.0),
        (1.0 / 1024.0, 0.0),
        (2.0 / 1024.0, 2.0 / 1024.0),
        (-1.0 / 1024.0, -2.0 / 1024.0),
    ] {
        assert_eq!(
            AxisLengths::before(&half, anchor).unwrap().to_bits(),
            f64::to_bits(expected)
        );
    }
}
