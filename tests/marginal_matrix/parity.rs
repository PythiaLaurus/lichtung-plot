use super::*;

#[test]
fn marginal_full_svg_fingerprints() {
    // T5b2a: guides use theme-owned typography/chrome. Historical pre-RGBA
    // Publication: (21377,23ab035bb3873cd5), (22431,10de42b68b4603e1),
    // (22379,04ea26232bec117e), (20238,3f3b8cdffa3cfdc4).
    // Science: (20930,0be95cd7ce3953a1), (22125,8717d641a8a8efc5),
    // (22074,f9534b68c21f3d55), (19917,ac5591db9566f8f8).
    for (theme, expected) in [
        (
            lichtung_plot::Theme::Publication,
            [
                (21407, 0xd98b_8667_3bf4_eda3),
                (22520, 0x4e6e_8fe7_8fff_b7e9),
                (22468, 0x37dc_e108_76a3_247e),
                (20298, 0x0b82_06e4_e0c7_15ea),
            ],
        ),
        (
            lichtung_plot::Theme::Science,
            [
                (20960, 0x1167_e8da_cd48_f613),
                (22214, 0x874e_499b_d3cb_5dcf),
                (22163, 0xee10_4c90_f944_01a5),
                (19977, 0x91ed_5cbf_bafc_9a36),
            ],
        ),
    ] {
        let mut observed = Vec::new();
        for (case, _) in expected.into_iter().enumerate() {
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
            observed.push((svg.len(), hash));
        }
        assert_eq!(observed, expected, "{theme:?}");
    }
}
