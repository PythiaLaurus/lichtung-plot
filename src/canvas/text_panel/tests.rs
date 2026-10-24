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

#[test]
fn assigned_sequence_height_fits_body_without_shrinking_an_independent_heading() {
    use crate::{TextFit, TextFitGrouping};
    let theme = crate::Theme::publication().resolve().unwrap();
    let heading = theme.typography.composition.text_panel_title.clone();
    let body = theme
        .typography
        .composition
        .body
        .clone()
        .font_size(14.25)
        .unwrap();
    let source = TextPanel::new(
        Text::new("Independent title").style(heading.clone()),
        Text::new("Complete first line\nComplete second line")
            .style(body.clone())
            .layout(
                TextLayout::default().fit(
                    TextFit::shrink_to(10.25)
                        .unwrap()
                        .grouping(TextFitGrouping::Uniform),
                ),
            ),
    )
    .unwrap();
    let title = theme
        .measure_text(
            source.title(),
            &heading,
            crate::layout::PhysicalTextAlignment::Start,
            "height witness title",
            false,
        )
        .unwrap();
    let floor = theme
        .measure_text(
            source.body(),
            &body.clone().font_size(11.5).unwrap(),
            crate::layout::PhysicalTextAlignment::Start,
            "independent requested body height",
            false,
        )
        .unwrap();
    let lengths = theme
        .text_panel_lengths(heading.get_font_size(), body.get_font_size())
        .unwrap();
    let height = lengths
        .inset
        .times(2)
        .add_length(&lengths.gap)
        .add_pixels(title.height() + floor.height())
        .unwrap()
        .pixels_rounded(crate::layout::kernel::SpatialRounding::Ceil)
        .unwrap();
    let intrinsic = TextPanelIntrinsics::prepare(&source, &theme).unwrap();
    let plan = fit::measured(TextPanelPlan::measure(
        &source,
        0,
        PanelBox::new(0.0, 400.0, 0.0, height),
        &theme,
        intrinsic,
        &FitScope::root(),
    ))
    .unwrap();
    assert_eq!(
        plan.title.plan().style.font_size.to_bits(),
        heading.get_font_size().to_bits()
    );
    assert!((10.25..14.25).contains(&plan.body.plan().style.font_size));
    assert_eq!(plan.body.plan().source, source.body());
    assert!(plan
        .body
        .plan()
        .fit_input
        .as_ref()
        .unwrap()
        .height
        .is_some());
    let footprint =
        crate::layout::text::placement::footprint(plan.body_placement, plan.body.plan()).unwrap();
    assert!(footprint.y0 >= 0.0 && footprint.y1 <= height);
}
