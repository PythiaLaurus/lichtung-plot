use super::*;

fn source() -> CompositionTheme {
    CompositionTheme::builtin(20, 15, 15, 17).unwrap()
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one exhaustive schema fixture covers every group and parent edit"
)]
fn all_relationship_edits_replace_one_field_and_preserve_immutable_siblings() {
    let source = source();
    let replacement = RelativeSpacing::ratio(7, 3).unwrap();
    macro_rules! group {
        ($group:ident; $($getter:ident => $edit:ident: $index:literal),+ $(,)?) => { $(
            let original = source.$group();
            let changed = original.clone().$edit(RelativeSpacing::ratio(9, 2).unwrap()).unwrap().$edit(replacement.clone()).unwrap();
            assert_eq!(changed.$getter(), &replacement);
            for (index, (left, right)) in original.relationships().zip(changed.relationships()).enumerate() {
                if index != $index { assert_eq!(left, right); }
            }
        )+ };
    }
    group!(root_title_block; title_subtitle_gap => with_title_subtitle_gap: 0,
        title_block_to_content_clearance => with_title_block_to_content_clearance: 1,
        caption_to_content_clearance => with_caption_to_content_clearance: 2);
    group!(root_padding; left => with_left: 0, right => with_right: 1, bottom => with_bottom: 2, top => with_top: 3);
    group!(text_panel; inset => with_inset: 0, gap => with_gap: 1);
    group!(headers; column_header_padding => with_column_header_padding: 0, row_header_padding => with_row_header_padding: 1,
        row_title_horizontal_padding => with_row_title_horizontal_padding: 2, row_title_vertical_padding => with_row_title_vertical_padding: 3);
    group!(facet_strip; horizontal_inset => with_horizontal_inset: 0, content_clearance => with_content_clearance: 1);
    macro_rules! parent {
        ($field:ident, $edit:ident, $value:expr) => {
            let value = $value;
            let changed = source.clone().$edit(value.clone()).unwrap();
            assert_eq!(changed.$field(), &value);
        };
    }
    parent!(
        root_title_block,
        with_root_title_block,
        source
            .root_title_block()
            .clone()
            .with_title_subtitle_gap(replacement.clone())
            .unwrap()
    );
    parent!(
        root_padding,
        with_root_padding,
        source
            .root_padding()
            .clone()
            .with_left(replacement.clone())
            .unwrap()
    );
    parent!(
        text_panel,
        with_text_panel,
        source
            .text_panel()
            .clone()
            .with_inset(replacement.clone())
            .unwrap()
    );
    parent!(
        headers,
        with_headers,
        source
            .headers()
            .clone()
            .with_row_header_padding(replacement.clone())
            .unwrap()
    );
    parent!(
        facet_strip,
        with_facet_strip,
        source
            .facet_strip()
            .clone()
            .with_content_clearance(replacement.clone())
            .unwrap()
    );
    parent!(
        panel_edge_clearance,
        with_panel_edge_clearance,
        replacement.clone()
    );
    parent!(facet_gap, with_facet_gap, replacement.clone());
    parent!(canvas_grid_gap, with_canvas_grid_gap, replacement);
    let changed = source
        .clone()
        .with_root_title_alignment(TitleAlignment::End)
        .unwrap()
        .with_root_subtitle_alignment(Some(TitleAlignment::Start))
        .unwrap()
        .with_root_caption_alignment(TitleAlignment::Center)
        .unwrap();
    assert_eq!(changed.root_title_alignment(), TitleAlignment::End);
    assert_eq!(
        changed.root_subtitle_alignment(),
        Some(TitleAlignment::Start)
    );
    assert_eq!(changed.root_caption_alignment(), TitleAlignment::Center);
    assert_eq!(changed.root_padding(), source.root_padding());
    assert_eq!(
        changed
            .with_root_subtitle_alignment(None)
            .unwrap()
            .root_subtitle_alignment(),
        None
    );
    assert_eq!(source.root_title_alignment(), TitleAlignment::Center);
}

#[test]
fn root_padding_keeps_fractional_edges_until_the_complete_hard_boundary() {
    use crate::layout::canvas::RootFramePlan;
    let mut theme = super::super::Theme::Publication.resolve().unwrap();
    let third = RelativeSpacing::ratio(1, 17 * 3072).unwrap();
    let zero = RelativeSpacing::ratio(0, 1).unwrap();
    theme.composition = theme
        .composition
        .clone()
        .with_root_padding(RootPadding::new(third.clone(), third, zero.clone(), zero).unwrap())
        .unwrap();
    let measure = |hard| {
        RootFramePlan::measure(
            (1, 1),
            &theme,
            [None; 3],
            crate::decorations::title_block::plan::RootAlignments::resolve(
                &theme, [None; 2], [None; 2],
            ),
            (hard, 1.0 / 1024.0),
        )
    };
    assert!(measure(1023.0 / 1024.0).is_ok());
    assert!(matches!(
        measure(1.0),
        Err(crate::layout::fit::PlanError::Fit(_))
    ));

    assert_eq!(
        theme
            .root_padding_lengths()
            .unwrap()
            .horizontal()
            .round(SpatialRounding::Ceil)
            .unwrap()
            .quanta(),
        1
    );
}
