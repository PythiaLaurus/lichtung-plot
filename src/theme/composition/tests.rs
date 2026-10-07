use super::*;

fn source() -> CompositionTheme {
    CompositionTheme::builtin(20, 15, 15, 17, &OperationGenerationResources::new()).unwrap()
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
                if index != $index { assert!(left.shares_payload_with(right)); }
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
            let mut expected = source.clone();
            expected.$field = value;
            assert_eq!(changed, expected);
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
fn complete_builtin_sources_have_exact_preset_local_basis_without_fonts() {
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    for (t, s, c, a) in [
        (20, 15, 15, 17),
        (19, 14, 14, 16),
        (21, 16, 15, 17),
        (19, 15, 15, 17),
    ] {
        let source =
            CompositionTheme::builtin(t, s, c, a, &OperationGenerationResources::new()).unwrap();
        let pairs = [
            (0, 1),
            (14, t.max(s)),
            (12, c),
            (16, a),
            (16, a),
            (8, a),
            (12, a),
            (4, 16),
            (16, a),
            (12, t.max(a)),
            (12, s),
            (12, a),
            (16, t),
            (20, t),
            (16, t),
            (8, t),
            (16, s),
            (8, s),
        ];
        assert_eq!(source.retained_relationships().count(), 18);
        for (value, (n, d)) in source.retained_relationships().zip(pairs) {
            assert_eq!(value, &RelativeSpacing::ratio(n, d).unwrap());
        }
    }
    for invalid in [f64::NAN, f64::INFINITY, -1.0] {
        assert!(RelativeSpacing::new(invalid).is_err());
    }
    assert!(source()
        .with_panel_edge_clearance(RelativeSpacing::new(-0.0).unwrap())
        .is_ok());
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
}

#[test]
fn builtin_admission_matches_independent_integer_certificate_and_refuses_before_fonts() {
    for (t, s, c, a, expected) in [
        (20, 15, 15, 17, 109),
        (19, 14, 14, 16, 106),
        (21, 16, 15, 17, 112),
        (19, 15, 15, 17, 117),
    ] {
        let (work, bytes) = builtin_resource_oracle(t, s, c, a);
        assert_eq!(work, expected);
        for (visits, limit) in [
            (0, bytes),
            (work - 1, bytes),
            (work, bytes - 1),
            (work, bytes),
        ] {
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
            let resources = OperationGenerationResources::with_test_limits(visits, limit);
            let result = CompositionTheme::builtin(t, s, c, a, &resources);
            assert_eq!(result.is_ok(), visits == work && limit == bytes);
            if result.is_ok() {
                assert_eq!(resources.usage().unwrap(), (0, work));
                assert_eq!(resources.byte_usage().unwrap(), bytes);
            }
            assert_eq!(
                crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
                0
            );
        }
    }
}

#[test]
fn identity_visits_all_actual_handles_and_alignment_mismatch_stops_before_ratios() {
    let source = source();
    for visits in [0, 18, 19] {
        let resources = OperationGenerationResources::with_test_limits(visits, 0);
        assert_eq!(
            source.same(&source.clone(), &resources).is_ok(),
            visits == 19
        );
        assert_eq!(resources.byte_usage().unwrap(), 0);
    }
    let resources = OperationGenerationResources::with_test_limits(1, 0);
    assert!(!source
        .same(
            &source
                .clone()
                .with_root_subtitle_alignment(Some(TitleAlignment::End))
                .unwrap(),
            &resources
        )
        .unwrap());
}

#[test]
fn root_padding_keeps_fractional_edges_until_the_complete_hard_boundary() {
    use crate::layout::canvas::RootFramePlan;
    let mut theme = super::super::Theme::Publication
        .resolve(&OperationGenerationResources::new())
        .unwrap();
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
    let resources = theme
        .prepared_font_environment()
        .unwrap()
        .generation_resources();
    assert_eq!(
        theme
            .root_padding_lengths()
            .unwrap()
            .horizontal(&resources)
            .unwrap()
            .round(SpatialRounding::Ceil, &resources)
            .unwrap()
            .quanta(),
        1
    );
}

#[test]
fn complete_local_metrics_ignore_legacy_carriers_and_preserve_distinct_gap_and_track_owners() {
    let mut theme = super::super::Theme::Publication
        .resolve(&OperationGenerationResources::new())
        .unwrap();
    theme.label_font_size = 1;
    theme.composition_title_font_size = 1;
    theme.facet_header_font_size = 1;
    assert_eq!(
        theme.composition_grid_gap().unwrap().to_bits(),
        12.0_f64.to_bits()
    );
    assert_eq!(
        theme.composition_facet_gap().unwrap().to_bits(),
        12.0_f64.to_bits()
    );
    theme.typography.composition.body = theme
        .typography
        .composition
        .body
        .clone()
        .with_font_size(34)
        .unwrap();
    assert_eq!(
        theme.composition_grid_gap().unwrap().to_bits(),
        24.0_f64.to_bits()
    );
    assert_eq!(
        theme.root_outer_width(0.0).unwrap().to_bits(),
        64.0_f64.to_bits()
    );
    assert_eq!(
        theme.typography.composition.composition_header.font_size(),
        20
    );
    theme.typography.composition.facet_header = theme
        .typography
        .composition
        .facet_header
        .clone()
        .with_font_size(30)
        .unwrap();
    assert_eq!(
        theme.composition_facet_gap().unwrap().to_bits(),
        24.0_f64.to_bits()
    );
    let resources = theme
        .prepared_font_environment()
        .unwrap()
        .generation_resources();
    assert_eq!(
        hard(
            &theme.facet_strip_lengths().unwrap().horizontal_inset,
            &resources
        )
        .unwrap()
        .to_bits(),
        32.0_f64.to_bits()
    );
}
