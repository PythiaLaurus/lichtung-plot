use super::*;

#[test]
fn marginal_full_svg_fingerprints() {
    // 2026-09-14 T5a6d: canonical projection/title midpoints and ink-aware
    // endpoint anchors. Historical 13ce394 fingerprints and inspected deltas are
    // in docs/local-search-benchmarks.md, "Final Numerical Follow-Up"; not byte parity.
    for (theme, expected) in [
        (
            lichtung_plot::Theme::Publication,
            [
                (21377, 0x23ab_035b_b387_3cd5),
                (22526, 0x1db2_ae0c_f480_f34f),
                (22698, 0x09fa_0b62_35a6_4714),
                (20238, 0x3f3b_8cdf_fa3c_fdc4),
            ],
        ),
        (
            lichtung_plot::Theme::Science,
            [
                (20930, 0x0be9_5cd7_ce39_53a1),
                (22222, 0x7ad7_81ba_0fb2_bde2),
                (22234, 0x7ab1_17b8_ed15_922c),
                (19917, 0xac55_91db_9566_f8f8),
            ],
        ),
    ] {
        for (case, expected) in expected.into_iter().enumerate() {
            let mut plot = Plot::marginal(
                NumericValues::new("x", [1.0, 1.2, 1.4, 3.0, 3.2, 3.4]),
                NumericValues::new("y", [2.0, 2.1, 2.3, 4.0, 4.2, 4.3]),
            )
            .theme(theme)
            .size(800, 600);
            if case > 0 {
                plot =
                    plot.hue(CategoryValues::new("hue", ["b", "b", "b", "a", "a", "a"]).unwrap());
            }
            if case > 1 {
                plot = plot.group(
                    CategoryValues::new("group", ["g1", "g1", "g1", "g2", "g2", "g2"]).unwrap(),
                );
            }
            if case == 3 {
                plot = plot
                    .x_limits(0.5, 4.0)
                    .y_limits(1.0, 5.0)
                    .legend(LegendPolicy::Hidden);
            }
            let directory = unique_dir("marginal-parity");
            let output = directory.join("plot.svg");
            plot.svg_preserve_text(&output).unwrap();
            let svg = std::fs::read(&output).unwrap();
            std::fs::remove_dir_all(directory).unwrap();
            let hash = svg.iter().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
                (hash ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3)
            });
            assert_eq!((svg.len(), hash), expected, "{theme:?} case {case}");
        }
    }
}
