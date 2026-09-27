use super::*;

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one cross-parent fixture retains all Waterfall theme and scale fingerprints"
)]
fn waterfall_full_svg_fingerprints() {
    // 2026-09-14 T5a6d: exact category bands, canonical title midpoints and
    // ink-aware endpoint anchors. Historical 0e8c344 fingerprints are preserved
    // in docs/local-search-benchmarks.md, "Final Numerical Follow-Up"; not byte parity.
    for (theme, expected) in [
        (
            lichtung_plot::Theme::Publication,
            [
                (11551, 0x03c5_6745_6919_5895),
                (13511, 0x8c52_1a56_5593_25be),
                (9228, 0x9f67_23dd_a047_eb4d),
                (19089, 0xf3a0_7b27_ce2b_290c),
                (20239, 0xc304_afbc_8257_b288),
                (19089, 0xf3a0_7b27_ce2b_290c),
                (20239, 0xc304_afbc_8257_b288),
                (21446, 0x55d8_9ed9_f883_597a),
                (11798, 0xc759_4392_037a_ceb4),
            ],
        ),
        (
            lichtung_plot::Theme::Science,
            [
                (11375, 0xcf25_95f0_e76f_69a4),
                (13295, 0x23e6_7f91_c3c8_05d9),
                (9092, 0x0333_b4a3_ca40_0784),
                (18487, 0xfad4_f91b_4e9b_3136),
                (19659, 0xe49d_3bac_4385_602a),
                (18487, 0xfad4_f91b_4e9b_3136),
                (19659, 0xe49d_3bac_4385_602a),
                (19689, 0x7e83_740e_794f_b006),
                (11628, 0x13c5_5f8d_1b5a_f071),
            ],
        ),
    ] {
        let mut observed = Vec::new();
        for (case, _) in expected.into_iter().enumerate() {
            let mut plot = Plot::waterfall(
                CategoryValues::new("step", ["gain", "loss", "gain", "loss"]).unwrap(),
                NumericValues::new("delta", [4.0, -2.0, -0.0, 1.0]),
                10.0,
            )
            .theme(theme)
            .size(801, 603);
            if case == 1 {
                plot = plot.x_limits(11.0, 14.0);
            }
            if case == 2 {
                plot = plot
                    .total_labels(lichtung_plot::WaterfallLabels::new("Initial", "Final").unwrap())
                    .size(533, 281);
            }
            let directory = tempfile::tempdir().unwrap();
            let output = directory.path().join("plot.svg");
            if (3..7).contains(&case) {
                let scales = [
                    lichtung_plot::FacetScales::Fixed,
                    lichtung_plot::FacetScales::FreeX,
                    lichtung_plot::FacetScales::FreeY,
                    lichtung_plot::FacetScales::Free,
                ][case - 3];
                plot.facet(
                    lichtung_plot::FacetSpec::column(
                        CategoryValues::new("panel", ["left", "left", "right", "right"]).unwrap(),
                    )
                    .scales(scales),
                )
                .size(1401, 603)
                .svg_preserve_text(&output)
                .unwrap();
            } else if case == 7 {
                lichtung_plot::Canvas::grid(1, 2)
                    .unwrap()
                    .size(1601, 603)
                    .panel(0, 0, plot)
                    .unwrap()
                    .panel(
                        0,
                        1,
                        Plot::waterfall(
                            CategoryValues::new("step", ["down", "up"]).unwrap(),
                            NumericValues::new("delta", [-4.0, 9.0]),
                            3.0,
                        )
                        .theme(theme),
                    )
                    .unwrap()
                    .x_range_group([(0, 0), (0, 1)])
                    .unwrap()
                    .svg_preserve_text(&output)
                    .unwrap();
            } else if case == 8 {
                plot.title("Cumulative change")
                    .svg_preserve_text(&output)
                    .unwrap();
            } else {
                plot.svg_preserve_text(&output).unwrap();
            }
            let svg = std::fs::read(&output).unwrap();
            let hash = svg.iter().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
                (hash ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3)
            });
            observed.push((svg.len(), hash));
        }
        assert_eq!(observed, expected, "{theme:?}");
    }
}
