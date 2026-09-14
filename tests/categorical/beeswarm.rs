use super::*;

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "parent fingerprints cover the complete supported composition and color matrix"
)]
fn beeswarm_parent_svg_fingerprints() {
    // 2026-09-14 T5a6d: canonical projection and ink-aware endpoint anchors.
    // Historical 0a41f1c fingerprints and inspected deltas are preserved in
    // docs/local-search-benchmarks.md, "Final Numerical Follow-Up"; not byte parity.
    for (theme, expected) in [
        (
            lichtung_plot::Theme::Publication,
            [
                (10946, 0x00cc_ae03_cf55_3497),
                (10386, 0x9bd2_3e7b_85fa_aaa7),
                (10961, 0x7fea_882a_8ad4_45d5),
                (10678, 0xbc27_3f88_bd85_46e3),
                (20932, 0x0908_10e0_78b4_41ab),
                (20944, 0x354e_1563_114b_c064),
                (20932, 0x0908_10e0_78b4_41ab),
                (20944, 0x354e_1563_114b_c064),
                (23546, 0x56ed_98f3_4890_4a0c),
                (16217, 0x20c4_1435_dc3f_1d2a),
            ],
        ),
        (
            lichtung_plot::Theme::Science,
            [
                (10808, 0xebba_8167_47ab_02d5),
                (10233, 0x1f7b_dc87_2e63_4a50),
                (10806, 0x54c0_2423_3ca0_aa1f),
                (10737, 0x0678_c536_f028_b9eb),
                (20462, 0x4d76_0b94_93f9_2a03),
                (20470, 0x105d_9e0c_e068_2c2d),
                (20462, 0x4d76_0b94_93f9_2a03),
                (20470, 0x105d_9e0c_e068_2c2d),
                (21838, 0x359c_a0e8_129d_5367),
                (15087, 0x6e84_deb3_6277_5888),
            ],
        ),
    ] {
        for (case, expected) in expected.into_iter().enumerate() {
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
            assert_eq!((svg.len(), hash), expected, "{theme:?} case {case}");
        }
    }
}
