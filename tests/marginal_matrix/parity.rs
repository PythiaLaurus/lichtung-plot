use super::*;

#[test]
fn marginal_full_svg_fingerprints() {
    // Full preserve-text SVG baselines from clean 13ce394, not normalized geometry.
    for (theme, expected) in [
        (
            lichtung_plot::Theme::Publication,
            [
                (21380, 0x9bcc_7857_4b22_cb3d),
                (22526, 0x1db2_ae0c_f480_f34f),
                (22698, 0x09fa_0b62_35a6_4714),
                (20234, 0x4739_1d40_cb35_05d0),
            ],
        ),
        (
            lichtung_plot::Theme::Science,
            [
                (20933, 0x55db_04f2_b575_2211),
                (22222, 0x7ad7_81ba_0fb2_bde2),
                (22234, 0x7ab1_17b8_ed15_922c),
                (19913, 0x2c3a_c60f_7bab_067e),
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
