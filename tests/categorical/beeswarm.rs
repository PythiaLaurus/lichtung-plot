use super::*;

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "parent fingerprints cover the complete supported composition and color matrix"
)]
fn beeswarm_parent_svg_fingerprints() {
    // T5b2a: theme-owned guide typography changes colorbar text and panel reserve.
    // Historical T5a fingerprints remain attributed in docs/local-search-benchmarks.md.
    for (theme, expected) in [
        (
            lichtung_plot::Theme::Publication,
            [
                (10895, 0x847a_52df_86c5_10ce),
                (11461, 0x59b6_3400_9660_be02),
                (10893, 0x62d6_0a85_d95c_6fb7),
                (10917, 0x4b7b_1a07_4443_da38),
                (21065, 0x1659_4bfb_e2f5_b2d5),
                (21073, 0x84ab_3035_cd57_8b29),
                (21065, 0x1659_4bfb_e2f5_b2d5),
                (21073, 0x84ab_3035_cd57_8b29),
                (23666, 0xb1ba_9122_5895_3c22),
                (16335, 0xd55d_76c4_1ccf_344c),
            ],
        ),
        (
            lichtung_plot::Theme::Science,
            [
                (10726, 0x4bc6_478a_014a_c817),
                (10149, 0xd1c7_c4c8_2625_ebf9),
                (10724, 0x31bd_1262_25e0_7ef2),
                (10777, 0xf8b5_e0f6_070a_ba21),
                (20272, 0x8c23_04d8_598f_132e),
                (20280, 0xedd6_ee9d_c8b1_6a3e),
                (20272, 0x8c23_04d8_598f_132e),
                (20280, 0xedd6_ee9d_c8b1_6a3e),
                (21706, 0xc57e_0178_3533_ae79),
                (15068, 0xe7e2_7758_a0f8_1372),
            ],
        ),
    ] {
        let mut observed = Vec::new();
        for (case, _) in expected.into_iter().enumerate() {
            let colors = match case {
                2 => vec![0.25; 12],
                3 => vec![
                    -f64::MAX,
                    f64::MAX,
                    0.0,
                    -1.0,
                    1.0,
                    1e-8,
                    0.0,
                    0.5,
                    0.5,
                    0.5,
                    0.5,
                    0.5,
                ],
                _ => vec![
                    -1.0, 1.0, 0.0, -1.0, 1.0, 1e-8, 0.0, 0.5, 0.5, 0.5, 0.5, 0.5,
                ],
            };
            let mut plot = Plot::beeswarm(
                CategoryValues::new(
                    "row_name",
                    [
                        "b_b", "a_a", "b_b", "a_a", "b_b", "a_a", "b_b", "a_a", "b_b", "b_b",
                        "b_b", "b_b",
                    ],
                )
                .unwrap(),
                NumericValues::new(
                    "x_value",
                    [-2.0, 2.0, 0.0, 1.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                ),
                NumericValues::new("heat_color", colors),
            )
            .theme(theme)
            .size(801, 603)
            .zero_line();
            if case == 1 {
                plot = plot
                    .color_limits(0.0, 0.5)
                    .x_limits(-1.0, 1.0)
                    .hide_point_outline();
            }
            if case == 3 {
                plot = plot.color_limits(-f64::MAX, f64::MAX);
            }
            let directory = unique_dir("beeswarm-parity");
            let output = directory.join("plot.svg");
            if (4..8).contains(&case) {
                let scales = [
                    lichtung_plot::FacetScales::Fixed,
                    lichtung_plot::FacetScales::FreeX,
                    lichtung_plot::FacetScales::FreeY,
                    lichtung_plot::FacetScales::Free,
                ][case - 4];
                plot.facet(
                    lichtung_plot::FacetSpec::column(
                        CategoryValues::new(
                            "panel",
                            [
                                "left", "left", "left", "left", "left", "left", "right", "right",
                                "right", "right", "right", "right",
                            ],
                        )
                        .unwrap(),
                    )
                    .scales(scales),
                )
                .size(1401, 603)
                .svg_preserve_text(&output)
                .unwrap();
            } else if case == 8 {
                lichtung_plot::Canvas::grid(1, 2)
                    .unwrap()
                    .size(1601, 603)
                    .panel(0, 0, plot.clone())
                    .unwrap()
                    .panel(
                        0,
                        1,
                        Plot::beeswarm(
                            CategoryValues::new("row_name", ["b_b", "a_a"]).unwrap(),
                            NumericValues::new("x_value", [-4.0, 4.0]),
                            NumericValues::new("heat_color", [0.0, 1.0]),
                        )
                        .theme(theme),
                    )
                    .unwrap()
                    .x_range_group([(0, 0), (0, 1)])
                    .unwrap()
                    .svg_preserve_text(&output)
                    .unwrap();
            } else if case == 9 {
                lichtung_plot::Canvas::grid(1, 2)
                    .unwrap()
                    .size(1601, 603)
                    .panel(0, 0, plot)
                    .unwrap()
                    .panel(
                        0,
                        1,
                        Plot::heatmap(
                            CategoryValues::new("column", ["one", "two"]).unwrap(),
                            CategoryValues::new("row", ["top", "top"]).unwrap(),
                            NumericValues::new("heat_color", [-1.0, 1.0]),
                        )
                        .theme(theme),
                    )
                    .unwrap()
                    .svg_preserve_text(&output)
                    .unwrap();
            } else {
                plot.svg_preserve_text(&output).unwrap();
            }
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
