use super::*;

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "parent fingerprints cover the complete supported composition and color matrix"
)]
fn beeswarm_parent_svg_fingerprints() {
    // T5b2a: theme-owned guide typography changes colorbar text and panel reserve.
    // Historical T5a fingerprints remain attributed in docs/local-search-benchmarks.md.
    // T5d3a replaces RGB-interpolated Beeswarm interiors with OKLab paint.
    // T5d3b3 pins sampled interior guide stops to the resolved mark colors.
    // T5c4's mixed Beeswarm/Heatmap Canvas uses actual child grid minima, so
    // that one assigned panel's SVG coordinates differ from the 80x60 floor.
    // T5e1c uses the native Heatmap frame and common measured continuous guide.
    for (theme, expected) in [
        (
            lichtung_plot::Theme::Publication,
            [
                (11157, 0xd79b_e5df_bd28_1929),
                (11629, 0xf5a8_15ac_1814_f884),
                (11061, 0x8d04_a0f3_3136_f07d),
                (11132, 0x75e6_7cef_62b2_2b03),
                (21280, 0x6770_76e0_fc41_9692),
                (21288, 0xfa4a_8d63_b80d_4c7e),
                (21280, 0x6770_76e0_fc41_9692),
                (21288, 0xfa4a_8d63_b80d_4c7e),
                (23956, 0xd2cc_0b0e_32f7_043b),
                (16638, 3_444_976_425_010_708_410),
            ],
        ),
        (
            lichtung_plot::Theme::Science,
            [
                (10988, 0xc0f0_976e_b991_5abc),
                (10317, 0x1b4e_5c52_36d4_1b27),
                (10892, 0xad7f_1fad_367b_e77c),
                (10992, 0x0986_428e_689b_3904),
                (20487, 0x9db8_a421_2d7e_c3e1),
                (20495, 0xe9af_9d4c_808a_8f2d),
                (20487, 0x9db8_a421_2d7e_c3e1),
                (20495, 0xe9af_9d4c_808a_8f2d),
                (21996, 0xfac9_8f95_3aad_1e94),
                (15462, 17_295_330_126_320_557_267),
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
