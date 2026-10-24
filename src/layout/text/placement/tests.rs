use super::*;
use crate::layout::PhysicalTextAlignment;

fn near(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1.0 / 1024.0,
        "{actual} != {expected}"
    );
}

#[test]
fn box_anchors_preserve_the_selected_block_point_independently_of_line_alignment() {
    let reference = Point2D::new(123.25, 87.75);
    let anchors = [
        (TextAnchor::top_left(), 0.0, 0.0),
        (TextAnchor::top_center(), 0.5, 0.0),
        (TextAnchor::top_right(), 1.0, 0.0),
        (TextAnchor::center_left(), 0.0, 0.5),
        (TextAnchor::center(), 0.5, 0.5),
        (TextAnchor::center_right(), 1.0, 0.5),
        (TextAnchor::bottom_left(), 0.0, 1.0),
        (TextAnchor::bottom_center(), 0.5, 1.0),
        (TextAnchor::bottom_right(), 1.0, 1.0),
        (TextAnchor::fractions(-0.25, 1.5).unwrap(), -0.25, 1.5),
    ];
    for (anchor, x, y) in anchors {
        assert_eq!(anchor, TextAnchor::fractions(x, y).unwrap());
        for alignment in [PhysicalTextAlignment::Start, PhysicalTextAlignment::End] {
            let plan = super::super::tests::plan("AAAA\nA", i64::MAX, alignment);
            let placement = at(&plan, reference, anchor, 0.0, false, None).unwrap();
            let left = placement.position.x
                - if alignment == PhysicalTextAlignment::End {
                    plan.width()
                } else {
                    0.0
                };
            let top = placement.position.y + plan.height() / 2.0;
            near(left + x * plan.width(), reference.x);
            near(top - y * plan.height(), reference.y);
        }
    }
}

#[test]
fn measured_baselines_and_fractional_offsets_use_the_declared_frame_once() {
    let plan = super::super::tests::plan("A\n\nA", i64::MAX, PhysicalTextAlignment::Start);
    let reference = Point2D::new(80.25, 90.5);
    for (anchor, baseline) in [
        (
            TextAnchor::first_baseline(0.0).unwrap(),
            plan.first_baseline_quanta.unwrap(),
        ),
        (
            TextAnchor::center_baseline(0.0).unwrap(),
            i64::midpoint(
                plan.first_baseline_quanta.unwrap(),
                plan.last_baseline_quanta.unwrap(),
            ),
        ),
    ] {
        let placement = at(&plan, reference, anchor, 0.0, false, None).unwrap();
        near(
            placement.position.y + plan.height() / 2.0
                - f64::from(i32::try_from(baseline).unwrap()) / 1024.0,
            reference.y,
        );
    }
    let anchor = TextAnchor::top_left();
    let unshifted = at(&plan, reference, anchor, 90.0, false, None).unwrap();
    let screen = at(
        &plan,
        reference,
        anchor,
        90.0,
        false,
        Some(crate::TextOffset::new(1.25, 2.5, TextOffsetFrame::Screen).unwrap()),
    )
    .unwrap();
    let local = at(
        &plan,
        reference,
        anchor,
        90.0,
        false,
        Some(crate::TextOffset::new(1.25, 2.5, TextOffsetFrame::TextLocal).unwrap()),
    )
    .unwrap();
    near(screen.position.x - unshifted.position.x, 1.25);
    near(screen.position.y - unshifted.position.y, -2.5);
    near(local.position.x - unshifted.position.x, -2.5);
    near(local.position.y - unshifted.position.y, -1.25);
}
