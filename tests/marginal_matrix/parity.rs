use super::*;

#[test]
fn marginal_full_svg_fingerprints() {
    // T5b2a: guides use theme-owned typography/chrome. Historical pre-RGBA
    // Publication: (21377,23ab035bb3873cd5), (22431,10de42b68b4603e1),
    // (22379,04ea26232bec117e), (20238,3f3b8cdffa3cfdc4).
    // Science: (20930,0be95cd7ce3953a1), (22125,8717d641a8a8efc5),
    // (22074,f9534b68c21f3d55), (19917,ac5591db9566f8f8).
    // T5e3 replaces fixed strips with linked native Grid tracks and frozen axes.
    for (theme, expected) in [
        (
            lichtung_plot::Theme::Publication,
            [
                (17250, 3_448_215_693_771_509_407),
                (23542, 15_831_453_960_194_808_926),
                (23703, 5_648_259_560_182_979_111),
                (22083, 10_458_550_954_392_447_284),
            ],
        ),
        (
            lichtung_plot::Theme::Science,
            [
                (16645, 9_688_600_673_189_664_411),
                (22986, 4_412_269_117_742_133_946),
                (23147, 17_816_395_301_305_601_153),
                (21475, 15_442_460_721_036_199_690),
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
