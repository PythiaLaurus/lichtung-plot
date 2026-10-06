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
fn frame_sides_validate_all_fifteen_present_choices_and_every_consuming_edit() {
    assert!(FrameSides::new(false, false, false, false).is_err());
    for mask in 1..16 {
        let flags = [mask & 1 != 0, mask & 2 != 0, mask & 4 != 0, mask & 8 != 0];
        let sides = FrameSides::new(flags[0], flags[1], flags[2], flags[3]).unwrap();
        assert_eq!(
            [sides.top(), sides.right(), sides.bottom(), sides.left()],
            flags
        );
        assert_eq!(
            sides.drawing_order(),
            [flags[2], flags[1], flags[0], flags[3]]
        );
    }
    let sides = FrameSides::new(true, true, true, true).unwrap();
    for (index, edited) in [
        sides.clone().with_top(false).unwrap(),
        sides.clone().with_right(false).unwrap(),
        sides.clone().with_bottom(false).unwrap(),
        sides.clone().with_left(false).unwrap(),
    ]
    .into_iter()
    .enumerate()
    {
        let mut expected = [true; 4];
        expected[index] = false;
        assert_eq!(
            [edited.top(), edited.right(), edited.bottom(), edited.left()],
            expected
        );
    }
    assert!(FrameSides::new(true, false, false, false)
        .unwrap()
        .with_top(false)
        .is_err());
    assert!(FrameSides::new(false, true, false, false)
        .unwrap()
        .with_right(false)
        .is_err());
    assert!(FrameSides::new(false, false, true, false)
        .unwrap()
        .with_bottom(false)
        .is_err());
    assert!(FrameSides::new(false, false, false, true)
        .unwrap()
        .with_left(false)
        .is_err());
    assert_eq!(sides, FrameSides::new(true, true, true, true).unwrap());
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
fn ordinary_and_cell_frames_are_independent_complete_roles_with_cross_sibling_validation() {
    let source = complete();
    let cell = source.cell_frame().unwrap();
    assert_eq!(cell.sides().drawing_order(), [true, false, false, true]);
    let no_ordinary = source.clone().with_frame(None).unwrap();
    assert_eq!(no_ordinary.cell_frame(), source.cell_frame());
    let no_cell = source.clone().with_cell_frame(None).unwrap();
    assert_eq!(no_cell.frame(), source.frame());
    let changed = source
        .clone()
        .with_cell_frame(Some(
            FrameTheme::new(
                FrameSides::new(true, true, true, true).unwrap(),
                stroke(2.5),
            )
            .unwrap(),
        ))
        .unwrap();
    assert_eq!(changed.frame(), source.frame());
    assert_eq!(changed.ticks(), source.ticks());
    assert_eq!(changed.grid(), source.grid());
    assert_eq!(changed.spacing(), source.spacing());
    assert_eq!(
        changed.cell_frame().unwrap().sides().drawing_order(),
        [true; 4]
    );
    assert_eq!(changed.cell_frame().unwrap().stroke(), &stroke(2.5));
    let mut poisoned = source.clone();
    poisoned.cell_frame.as_mut().unwrap().stroke.width = 0.0;
    assert!(poisoned.clone().with_frame(None).is_err());
    assert!(poisoned.clone().with_ticks(None).is_err());
    assert!(poisoned.clone().with_grid(None).is_err());
    assert!(poisoned.clone().with_spacing(spacing()).is_err());
    assert!(AxisTheme::new(
        poisoned.frame,
        poisoned.cell_frame,
        poisoned.ticks,
        poisoned.grid,
        poisoned.spacing
    )
    .is_err());
    let mut poisoned = source.clone();
    poisoned.frame.as_mut().unwrap().stroke.width = f32::NAN;
    assert!(poisoned.with_cell_frame(None).is_err());
    let resources = OperationGenerationResources::with_test_limits(1, 0);
    assert!(!source.same(&changed, &resources).unwrap());
    assert_eq!(resources.usage().unwrap(), (0, 1));
    assert_eq!(resources.byte_usage().unwrap(), 0);
}

#[test]
fn all_six_spacing_edits_are_independent_and_resolve_against_their_selected_role() {
    let source = spacing();
    let replacement = RelativeSpacing::ratio(0, 1).unwrap();
    macro_rules! edit {
        ($field:ident, $method:ident, $index:expr) => {
            let edited = source
                .clone()
                .$method(RelativeSpacing::ratio(7, 3).unwrap())
                .unwrap()
                .$method(replacement.clone())
                .unwrap();
            assert_eq!(edited.$field(), &replacement);
            for (index, (left, right)) in source
                .relationships()
                .into_iter()
                .zip(edited.relationships())
                .enumerate()
            {
                if index == $index {
                    assert!(right.is_zero());
                } else {
                    assert!(left.shares_payload_with(right));
                }
            }
        };
    }
    edit!(x_tick_label_gap, with_x_tick_label_gap, 0);
    edit!(y_tick_label_gap, with_y_tick_label_gap, 1);
    edit!(x_title_tick_label_gap, with_x_title_tick_label_gap, 2);
    edit!(
        x_title_without_tick_labels_gap,
        with_x_title_without_tick_labels_gap,
        3
    );
    edit!(y_title_tick_label_gap, with_y_title_tick_label_gap, 4);
    edit!(y_title_edge_padding, with_y_title_edge_padding, 5);
    for (tick, title, factor) in [(16, 17, 1.0_f64), (32, 34, 2.0)] {
        let resources = OperationGenerationResources::new();
        let lengths = source.lengths(tick, title, &resources).unwrap();
        for (length, expected) in [
            &lengths.x_tick_label_gap,
            &lengths.y_tick_label_gap,
            &lengths.x_title_tick_label_gap,
            &lengths.x_title_without_tick_labels_gap,
            &lengths.y_title_tick_label_gap,
            &lengths.y_title_edge_padding,
        ]
        .into_iter()
        .zip([8.0, 6.0, 8.0, 16.0, 16.0, 12.0])
        {
            assert_eq!(
                AxisLengths::position(length, &resources).unwrap().to_bits(),
                (expected * factor).to_bits()
            );
            assert_eq!(
                AxisLengths::hard(length, &resources).unwrap().to_bits(),
                (expected * factor).to_bits()
            );
        }
    }
    assert!(source
        .lengths(0, 17, &OperationGenerationResources::new())
        .is_err());
    assert!(source
        .lengths(16, 0, &OperationGenerationResources::new())
        .is_err());
}

#[test]
fn complete_hard_sums_keep_fractional_gaps_until_the_single_rounding_boundary() {
    let resources = OperationGenerationResources::new();
    let first = RelativeSpacing::ratio(1, 3072)
        .unwrap()
        .product(LocalMetric::css_pixels(1).unwrap(), &resources)
        .unwrap();
    let sum = first
        .add(&first, &resources)
        .unwrap()
        .add(&first, &resources)
        .unwrap()
        .add_pixels(1.0, &resources)
        .unwrap();
    assert_eq!(
        AxisLengths::hard(&sum, &resources).unwrap().to_bits(),
        (1025.0_f64 / 1024.0).to_bits()
    );
    assert_eq!(
        AxisLengths::position(&first, &resources).unwrap().to_bits(),
        0
    );
    assert_eq!(
        AxisLengths::hard(&first, &resources).unwrap().to_bits(),
        (1.0_f64 / 1024.0).to_bits()
    );
    let huge = ExactLength::pixels(f64::MAX, &resources).unwrap();
    assert!(AxisLengths::position(&huge, &resources).is_err());
}

#[test]
fn whole_positions_round_half_quantum_ties_with_the_final_anchor_parity() {
    let resources = OperationGenerationResources::new();
    let half = RelativeSpacing::ratio(1, 2048)
        .unwrap()
        .product(LocalMetric::css_pixels(1).unwrap(), &resources)
        .unwrap();
    for (anchor, expected) in [
        (0.0, 0.0),
        (1.0 / 1024.0, 0.0),
        (2.0 / 1024.0, 2.0 / 1024.0),
        (-1.0 / 1024.0, -2.0 / 1024.0),
    ] {
        assert_eq!(
            AxisLengths::before(&half, anchor, &resources)
                .unwrap()
                .to_bits(),
            f64::to_bits(expected)
        );
    }
}

#[test]
fn axis_sharing_and_full_identity_have_exact_handle_and_content_refusal_boundaries() {
    let source = complete();
    for remaining in [0, 5, 6] {
        let resources = OperationGenerationResources::with_test_limits(remaining, 0);
        let shared = source.share(&resources);
        assert_eq!(shared.is_ok(), remaining == 6);
        assert_eq!(resources.byte_usage().unwrap(), 0);
        if let Ok(shared) = shared {
            assert_eq!(resources.usage().unwrap(), (0, 6));
            for (left, right) in source
                .spacing()
                .relationships()
                .into_iter()
                .zip(shared.spacing().relationships())
            {
                assert!(left.shares_payload_with(right));
            }
        }
    }
    let shared = source.clone();
    let independent = complete();
    for (other, work) in [(&shared, 7), (&independent, 31)] {
        for remaining in [0, work - 1, work] {
            let resources = OperationGenerationResources::with_test_limits(remaining, 0);
            let result = source.same(other, &resources);
            assert_eq!(result.is_ok(), remaining == work);
            if let Ok(same) = result {
                assert!(same);
            }
            assert_eq!(resources.byte_usage().unwrap(), 0);
        }
    }
    let changed = source
        .clone()
        .with_spacing(
            source
                .spacing()
                .clone()
                .with_y_title_edge_padding(RelativeSpacing::ratio(1, 1).unwrap())
                .unwrap(),
        )
        .unwrap();
    for remaining in [8, 9] {
        let resources = OperationGenerationResources::with_test_limits(remaining, 0);
        let result = source.same(&changed, &resources);
        if remaining == 9 {
            assert!(!result.unwrap());
        } else {
            assert!(result.is_err());
        }
        assert_eq!(resources.usage().unwrap(), (0, remaining));
    }
}
