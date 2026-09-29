use super::*;

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one cross-parent fixture retains all Waterfall theme and scale fingerprints"
)]
fn waterfall_full_svg_fingerprints() {
    // 2026-09-14 T5a6d: exact category bands, canonical title midpoints and
    // ink-aware endpoint anchors. T5c3b3 retains exact projected total targets
    // instead of clamping their text; the former hashes below were measured
    // with shifted base/terminal labels. Earlier 0e8c344 fingerprints are in
    // docs/local-search-benchmarks.md, "Final Numerical Follow-Up".
    for (theme, expected) in [
        (
            lichtung_plot::Theme::Publication,
            [
                (11525, 0xef4d_a9c5_54b7_1f07),
                (13511, 0x8c52_1a56_5593_25be),
                (9231, 0x3f6e_d3d6_1d3f_507b),
                (18953, 0x32b7_29a3_7300_ff5f),
                (20141, 0x58ba_3a96_c5c5_fa72),
                (18953, 0x32b7_29a3_7300_ff5f),
                (20141, 0x58ba_3a96_c5c5_fa72),
                (21446, 0x55d8_9ed9_f883_597a),
                (11772, 0x7e1c_8b4d_1d6b_414a),
            ],
        ),
        (
            lichtung_plot::Theme::Science,
            [
                (11349, 0xbc22_4d52_fb08_b5f2),
                (13295, 0x23e6_7f91_c3c8_05d9),
                (9095, 0xd141_38b2_83cb_528e),
                (18525, 0x3616_cd0e_ab21_bae8),
                (19660, 0xdba7_005e_6830_1dd5),
                (18525, 0x3616_cd0e_ab21_bae8),
                (19660, 0xdba7_005e_6830_1dd5),
                (19689, 0x1e96_01a6_7e20_a1ef),
                (11602, 0x17d8_1f43_ae05_5499),
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
