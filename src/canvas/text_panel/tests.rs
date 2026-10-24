use super::*;
use crate::{Text, TextAnchor, TextLayout, TextOffset, TextOffsetFrame};

#[test]
fn body_anchor_uses_the_enlarged_post_inset_sequence_cell_not_the_whole_panel() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let source = TextPanel::new(
        "Title",
        Text::new("A body\nwith two lines").layout(
            TextLayout::default()
                .alignment(TextAlignment::End)
                .anchor(TextAnchor::center())
                .offset(TextOffset::new(0.25, 0.5, TextOffsetFrame::Screen).unwrap()),
        ),
    )
    .unwrap();
    let intrinsic = TextPanelIntrinsics::prepare(&source, &theme).unwrap();
    let roles = source.appearances(&theme);
    let spacing = theme
        .text_panel_lengths(
            roles[0].theme.get_font_size(),
            roles[1].theme.get_font_size(),
        )
        .unwrap();
    let inset = spacing
        .inset
        .pixels_rounded(crate::layout::kernel::SpatialRounding::NearestEven)
        .unwrap();
    let gap = spacing
        .gap
        .pixels_rounded(crate::layout::kernel::SpatialRounding::NearestEven)
        .unwrap();
    for cell in [
        PanelBox::new(40.0, 440.0, 30.0, 430.0),
        PanelBox::new(40.0, 640.0, 30.0, 730.0),
    ] {
        let plan = fit::measured(TextPanelPlan::measure(
            &source,
            0,
            cell,
            &theme,
            intrinsic,
            &FitScope::root(),
        ))
        .unwrap();
        let x = f64::midpoint(cell.x0 + inset, cell.x1 - inset) + 0.25;
        let top = cell.y1 - inset - plan.title.height() - gap;
        let y = f64::midpoint(cell.y0 + inset, top) - 0.5;
        assert!(
            (plan.body_placement.position.x - plan.body.plan().width() / 2.0 - x).abs()
                <= 1.0 / 1024.0
        );
        assert!((plan.body_placement.position.y - y).abs() <= 1.0 / 1024.0);
        let bounds =
            crate::layout::text::placement::footprint(plan.body_placement, plan.body.plan())
                .unwrap();
        assert!(
            bounds.x0 >= cell.x0 + inset
                && bounds.x1 <= cell.x1 - inset
                && bounds.y0 >= cell.y0 + inset
                && bounds.y1 <= top
        );
    }
}
