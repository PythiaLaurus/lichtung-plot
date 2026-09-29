use super::*;

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "parent fingerprints cover the complete supported composition and color matrix"
)]
fn beeswarm_parent_svg_fingerprints() {
    // T5b2a: theme-owned guide typography changes colorbar text and panel reserve.
    // Historical T5a fingerprints remain attributed in docs/local-search-benchmarks.md.
    // T5c4's mixed Beeswarm/Heatmap Canvas uses actual child grid minima, so
    // that one assigned panel's SVG coordinates differ from the 80x60 floor.
    for (theme, expected) in [
        (
            lichtung_plot::Theme::Publication,
            [
                (11063, 0xbd34_8463_eec1_53ba),
                (11629, 0xf5a8_15ac_1814_f884),
                (11061, 0xc828_4045_d9d9_ab3f),
                (11085, 0x50be_b512_009e_4a1e),
                (21233, 0x613f_83b9_7b13_1575),
                (21241, 0xcd48_fc12_9662_7449),
                (21233, 0x613f_83b9_7b13_1575),
                (21241, 0xcd48_fc12_9662_7449),
                (23862, 0x3d61_a76a_7a25_b066),
                (16488, 0x0ac5_3bb8_1b0f_febb),
            ],
        ),
        (
            lichtung_plot::Theme::Science,
            [
                (10894, 0xcbcb_5c06_aff5_7ba9),
                (10317, 0x1b4e_5c52_36d4_1b27),
                (10892, 0xd0b3_c20f_ef32_5484),
                (10945, 0x3887_8655_2083_801f),
                (20440, 0x35d8_33e9_c136_51e4),
                (20448, 0x1f05_0879_c892_c62c),
                (20440, 0x35d8_33e9_c136_51e4),
                (20448, 0x1f05_0879_c892_c62c),
                (21902, 0x32ae_dbe6_a64e_54f7),
                (15148, 0xd534_82ca_b74f_83d9),
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
