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
    // T5c4's mixed Beeswarm/Heatmap Canvas uses actual child grid minima, so
    // that one assigned panel's SVG coordinates differ from the 80x60 floor.
    for (theme, expected) in [
        (
            lichtung_plot::Theme::Publication,
            [
                (11063, 0x493c_6a5d_fa6e_8635),
                (11629, 0xf5a8_15ac_1814_f884),
                (11061, 0x8d04_a0f3_3136_f07d),
                (11085, 0x7472_5c73_bca7_d746),
                (21233, 0xad2c_97f3_ae2a_5c1f),
                (21241, 0xa4cd_4dda_d918_dfeb),
                (21233, 0xad2c_97f3_ae2a_5c1f),
                (21241, 0xa4cd_4dda_d918_dfeb),
                (23862, 0xcb4e_caed_5eb3_e187),
                (16488, 0xd40b_f7d9_ef52_9f16),
            ],
        ),
        (
            lichtung_plot::Theme::Science,
            [
                (10894, 0xb83e_5cd5_433e_8eb0),
                (10317, 0x1b4e_5c52_36d4_1b27),
                (10892, 0xad7f_1fad_367b_e77c),
                (10945, 0x0db9_5241_6176_76f1),
                (20440, 0x0d74_7848_fa27_24d6),
                (20448, 0x346c_7f95_fd20_ecb6),
                (20440, 0x0d74_7848_fa27_24d6),
                (20448, 0x346c_7f95_fd20_ecb6),
                (21902, 0x70b9_f266_0348_c7f0),
                (15148, 0xc2d8_1b5b_b230_80b0),
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
