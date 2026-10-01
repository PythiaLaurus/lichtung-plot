use super::*;

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one cross-parent fixture retains all Waterfall theme and scale fingerprints"
)]
fn waterfall_full_svg_fingerprints() {
    // 2026-09-14 T5a6d: exact category bands, canonical title midpoints and
    // ink-aware endpoint anchors. T5c3b3 retains exact projected total targets
    // instead of clamping their text; the former hashes below were measured
    // with shifted base/terminal labels. Earlier 0e8c344 fingerprints are in
    // docs/local-search-benchmarks.md, "Final Numerical Follow-Up".
    // T5e5 uses native category tracks and exact baseline/ink shelves.
    for (theme, expected) in [
        (
            lichtung_plot::Theme::Publication,
            [
                (11176, 12_620_242_395_058_704_048),
                (12986, 11_002_164_979_684_030_776),
                (8737, 16_849_335_591_783_587_863),
                (19009, 17_390_987_378_678_253_197),
                (20201, 9_536_499_516_771_438_628),
                (19009, 17_390_987_378_678_253_197),
                (20201, 9_536_499_516_771_438_628),
                (21398, 18_376_741_918_976_031_177),
                (11419, 10_334_045_381_399_783_827),
            ],
        ),
        (
            lichtung_plot::Theme::Science,
            [
                (10713, 14_370_975_614_463_528_702),
                (12395, 15_352_510_697_281_771_398),
                (8402, 10_122_079_799_411_587_042),
                (18472, 10_218_387_039_860_752_139),
                (19563, 17_738_419_542_591_256_672),
                (18472, 10_218_387_039_860_752_139),
                (19563, 17_738_419_542_591_256_672),
                (19688, 2_415_462_878_694_278_945),
                (10940, 5_369_764_560_102_802_678),
            ],
        ),
    ] {
        let mut observed = Vec::new();
        for (case, _) in expected.into_iter().enumerate() {
            let mut plot = Plot::waterfall(
                CategoryValues::new("step", ["gain", "loss", "gain", "loss"]).unwrap(),
                NumericValues::new("delta", [4.0, -2.0, -0.0, 1.0]),
                10.0,
            )
            .theme(theme)
            .size(801, 603);
            if case == 1 {
                plot = plot.x_limits(11.0, 14.0);
            }
            if case == 2 {
                plot = plot
                    .total_labels(lichtung_plot::WaterfallLabels::new("Initial", "Final").unwrap())
                    .size(533, 281);
            }
            let directory = tempfile::tempdir().unwrap();
            let output = directory.path().join("plot.svg");
            if (3..7).contains(&case) {
                let scales = [
                    lichtung_plot::FacetScales::Fixed,
                    lichtung_plot::FacetScales::FreeX,
                    lichtung_plot::FacetScales::FreeY,
                    lichtung_plot::FacetScales::Free,
                ][case - 3];
                plot.facet(
                    lichtung_plot::FacetSpec::column(
                        CategoryValues::new("panel", ["left", "left", "right", "right"]).unwrap(),
                    )
                    .scales(scales),
                )
                .size(1401, 603)
                .svg_preserve_text(&output)
                .unwrap();
            } else if case == 7 {
                lichtung_plot::Canvas::grid(1, 2)
                    .unwrap()
                    .size(1601, 603)
                    .panel(0, 0, plot)
                    .unwrap()
                    .panel(
                        0,
                        1,
                        Plot::waterfall(
                            CategoryValues::new("step", ["down", "up"]).unwrap(),
                            NumericValues::new("delta", [-4.0, 9.0]),
                            3.0,
                        )
                        .theme(theme),
                    )
                    .unwrap()
                    .x_range_group([(0, 0), (0, 1)])
                    .unwrap()
                    .svg_preserve_text(&output)
                    .unwrap();
            } else if case == 8 {
                plot.title("Cumulative change")
                    .svg_preserve_text(&output)
                    .unwrap();
            } else {
                plot.svg_preserve_text(&output).unwrap();
            }
            let svg = std::fs::read(&output).unwrap();
            let hash = svg.iter().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
                (hash ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3)
            });
            observed.push((svg.len(), hash));
        }
        assert_eq!(observed, expected, "{theme:?}");
    }
}
