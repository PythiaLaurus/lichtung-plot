use super::*;

#[test]
fn waterfall_full_svg_fingerprints() {
    // 2026-09-14 T5a6d: exact category bands, canonical title midpoints and
    // ink-aware endpoint anchors. Historical 0e8c344 fingerprints are preserved
    // in docs/local-search-benchmarks.md, "Final Numerical Follow-Up"; not byte parity.
    for (theme, expected) in [
        (
            lichtung_plot::Theme::Publication,
            [
                (11631, 0xca35_95c4_f274_732f),
                (13591, 0x0a70_69bb_87d3_3258),
                (9308, 0x9c74_f6fa_6420_1727),
                (19169, 0x0bfb_fd11_7fae_60f4),
                (20319, 0xeb2a_3dac_0d7b_b178),
                (19169, 0x0bfb_fd11_7fae_60f4),
                (20319, 0xeb2a_3dac_0d7b_b178),
                (21510, 0xaa46_5388_a0fc_596f),
                (11878, 0xa930_324a_5cd7_c27e),
            ],
        ),
        (
            lichtung_plot::Theme::Science,
            [
                (11455, 0xe980_0eea_d1f7_ecf6),
                (13375, 0x0c7b_4ce2_57fa_f4a3),
                (9172, 0x683a_31ed_45db_1dbe),
                (18567, 0xa130_fea7_c8fe_99c2),
                (19739, 0x38e4_d759_cc83_e1ba),
                (18567, 0xa130_fea7_c8fe_99c2),
                (19739, 0x38e4_d759_cc83_e1ba),
                (19753, 0x34d4_4d64_988d_d71b),
                (11708, 0xf23a_b507_5d3a_2591),
            ],
        ),
    ] {
        for (case, expected) in expected.into_iter().enumerate() {
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
            assert_eq!((svg.len(), hash), expected, "{theme:?} case {case}");
        }
    }
}
