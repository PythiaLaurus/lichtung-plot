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
    // T5e4 replaces bins with source-ordered tangent packing and native bands/guides.
    // T5e7/8 retains complete measured guide chrome and canonical call order.
    // Final re-audit shares the modern unique numeric grammar with Shared guides.
    // The pre-fix T5e4 fingerprints are attributed in docs/t5e7-guide-cache-closure.md.
    for (theme, expected) in [
        (
            lichtung_plot::Theme::Publication,
            [
                (11011, 7_711_629_207_207_043_693),
                (10308, 17_497_074_417_148_050_456),
                (10712, 7_876_384_501_172_559_051),
                (10990, 1_964_448_113_547_353_923),
                (20584, 4_246_893_857_668_627_387),
                (20596, 1_366_786_897_088_778_236),
                (20584, 4_246_893_857_668_627_387),
                (20596, 1_366_786_897_088_778_236),
                (23472, 8_634_305_464_358_583_039),
                (16401, 1_897_838_009_937_524_425),
            ],
        ),
        (
            lichtung_plot::Theme::Science,
            [
                (10612, 8_085_217_306_636_702_031),
                (9909, 3_500_028_301_995_356_182),
                (10313, 7_739_493_714_440_421_121),
                (10591, 7_280_213_351_892_729_749),
                (19297, 4_614_873_321_729_850_775),
                (19304, 1_279_663_164_562_033_713),
                (19297, 4_614_873_321_729_850_775),
                (19304, 1_279_663_164_562_033_713),
                (21752, 18_032_327_114_057_004_252),
                (15343, 9_498_746_522_630_476_693),
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
