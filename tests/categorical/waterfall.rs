use super::*;

#[test]
fn waterfall_full_svg_fingerprints() {
    // Complete preserve-text output from the combined renderer at clean 0e8c344.
    for (theme, expected) in [
        (
            lichtung_plot::Theme::Publication,
            [
                (11635, 0x39b8_9057_ec91_b04b),
                (13598, 0x8f32_9eda_9944_aa4b),
                (9303, 0x268a_af9a_e942_bc06),
                (19169, 0x0bfb_fd11_7fae_60f4),
                (20319, 0xeb2a_3dac_0d7b_b178),
                (19169, 0x0bfb_fd11_7fae_60f4),
                (20319, 0xeb2a_3dac_0d7b_b178),
                (21510, 0x349a_cc3e_0230_408e),
                (11882, 0x7ec8_c493_f743_7d22),
            ],
        ),
        (
            lichtung_plot::Theme::Science,
            [
                (11459, 0x70f3_6f87_8cd8_64b2),
                (13382, 0x8583_7f29_0de5_5d6a),
                (9167, 0x0e24_8ebb_cbea_cf4f),
                (18567, 0xa130_fea7_c8fe_99c2),
                (19739, 0x38e4_d759_cc83_e1ba),
                (18567, 0xa130_fea7_c8fe_99c2),
                (19739, 0x38e4_d759_cc83_e1ba),
                (19759, 0x6368_8b07_9b6c_7e4a),
                (11703, 0xe6fb_3656_ea16_f6cc),
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
