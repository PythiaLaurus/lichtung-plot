use super::*;

#[test]
fn marginal_full_svg_fingerprints() {
    // T5b2a: visible guides use theme-owned typography and chrome; hidden and
    // absent guides retain their T5a fingerprints attributed in the benchmarks.
    for (theme, expected) in [
        (
            lichtung_plot::Theme::Publication,
            [
                (21377, 0x23ab_035b_b387_3cd5),
                (22431, 0x10de_42b6_8b46_03e1),
                (22379, 0x04ea_2623_2bec_117e),
                (20238, 0x3f3b_8cdf_fa3c_fdc4),
            ],
        ),
        (
            lichtung_plot::Theme::Science,
            [
                (20930, 0x0be9_5cd7_ce39_53a1),
                (22125, 0x8717_d641_a8a8_efc5),
                (22074, 0xf953_4b68_c21f_3d55),
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
