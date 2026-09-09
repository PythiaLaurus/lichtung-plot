use super::*;

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "the matrix verifies cold, payload-hit, and reflow work under one unchanged root budget"
)]
fn typography_performance_evidence_and_root_cache_budgets() {
    let sentence = "Measured responses remained stable across independent trials under controlled conditions. ";
    let mut workloads = vec![
        ("original144".to_owned(), "A ".repeat(144)),
        ("caption200".to_owned(), sentence.repeat(20)),
        (
            "paragraphs200".to_owned(),
            [sentence.repeat(7), sentence.repeat(7), sentence.repeat(6)].join("\n"),
        ),
    ];
    for count in [128, 256, 512, 2048] {
        workloads.push((format!("words{count}"), "Measured ".repeat(count)));
    }
    let mut scaling = Vec::new();
    for (name, source) in workloads {
        let setup = std::time::Instant::now();
        let theme = crate::Theme::Publication.resolve();
        let environment = theme.prepared_font_environment().unwrap();
        let resources = canonical::OperationGenerationResources::new();
        environment
            .share_generation_resources(resources.clone())
            .unwrap();
        eprintln!(
            "{name} setup_us={} initial={:?} stack={}",
            setup.elapsed().as_micros(),
            environment.resource_usage().unwrap(),
            theme.typography.caption.stack
        );
        let mut first: Option<ResolvedTextPlan> = None;
        let mut cold_work = 0;
        for (phase, width) in [
            ("cold", 768.0),
            ("hit", 768.0),
            ("new_width", 736.0),
            ("return_width", 768.0),
        ] {
            let before = resources.usage().unwrap();
            let font_before = environment.resource_usage().unwrap();
            TEXT_WORK.with(|work| work.set(TextWork::default()));
            PARAGRAPH_BIDI_CALLS.with(|calls| calls.set(0));
            canonical::FONT_SHAPE_CALLS.with(|calls| calls.set(0));
            let start = std::time::Instant::now();
            let result = theme.plan_text(
                &source,
                &theme.typography.caption,
                width,
                PhysicalTextAlignment::Start,
                "performance caption",
                false,
            );
            let elapsed = start.elapsed();
            let after = resources.usage().unwrap();
            let font_after = environment.resource_usage().unwrap();
            let work = TEXT_WORK.with(std::cell::Cell::get);
            eprintln!("{name}/{phase}: elapsed_us={} slots={} work={} cumulative={after:?} bytes={} font_work={} {work:?} bidi={} shapes={}", elapsed.as_micros(), after.0 - before.0, after.1 - before.1, font_after.font_bytes - font_before.font_bytes, font_after.logical_work - font_before.logical_work, PARAGRAPH_BIDI_CALLS.with(std::cell::Cell::get), canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get));
            let plan = match result {
                Ok(plan) => plan,
                Err(error) => {
                    eprintln!("{name}/{phase}: refused: {error}");
                    assert!(
                        name == "words2048",
                        "ordinary cold and changed-width workloads must fit the same root budget"
                    );
                    assert!(error
                        .to_string()
                        .contains("candidate-generation work units limit of 1000000"));
                    assert!(after.1 <= 1_000_000 && after.1 > 990_000);
                    break;
                }
            };
            assert_eq!(plan.line_texts().concat(), source.replace('\n', ""));
            assert!(
                work.shaping_scalars <= 4 * source.chars().count(),
                "{name}: {work:?}"
            );
            assert!(
                work.exact_trials <= 2 * plan.lines.len() + 1,
                "{name}: {work:?}"
            );
            assert!(
                work.proposal_visits <= source.split_whitespace().count() + plan.lines.len(),
                "{name}: {work:?}"
            );
            let glyphs: usize = plan
                .lines
                .iter()
                .flat_map(|line| &line.runs)
                .map(|run| run.glyphs.len())
                .sum();
            eprintln!(
                "{name}/{phase}: lines={} retained_glyphs={glyphs} primary={:?}",
                plan.lines.len(),
                plan.lines[0].runs[0].face
            );
            assert!(after.1 <= 1_000_000);
            if phase == "hit" || phase == "return_width" {
                assert!(first.as_ref().unwrap().shares_payload(&plan));
                assert!(first.as_ref().unwrap().shares_analysis(&plan));
                assert_eq!(work, TextWork::default());
                assert_eq!(after.0, before.0);
                assert_eq!(font_after.font_bytes, font_before.font_bytes);
                let key_bytes = source.len()
                    + theme.typography.caption.stack.len()
                    + "performance caption".len();
                // One SHA-256 source pass (58 encoded scalar/header bytes),
                // binary search in a one/two-entry index, one digest equality,
                // and one full-key equality. No shaping-bucket scan on a hit.
                let binary_comparisons = if phase == "hit" { 1 } else { 2 };
                assert_eq!(
                    after.1 - before.1,
                    (key_bytes + 58 + binary_comparisons + 1 + key_bytes + 1) as u128
                );
            } else if phase == "cold" {
                cold_work = after.1 - before.1;
                first = Some(plan);
            } else {
                let first = first.as_ref().unwrap();
                assert!(!first.shares_payload(&plan));
                assert!(first.shares_analysis(&plan));
                PARAGRAPH_BIDI_CALLS.with(|calls| assert_eq!(calls.get(), 0));
                assert!(work.shaping_scalars <= 3 * source.chars().count());
                assert!(after.1 - before.1 < cold_work / 2);
                if let Some(words) = name.strip_prefix("words") {
                    if let Ok(words @ (128 | 256 | 512)) = words.parse::<usize>() {
                        scaling.push((words, cold_work, after.1 - before.1));
                    }
                }
            }
        }
    }
    assert_eq!(scaling.len(), 3);
    for pair in scaling.windows(2) {
        assert_eq!(pair[1].0, pair[0].0 * 2);
        assert!(pair[1].1 <= pair[0].1 * 2 + 4096, "cold work: {scaling:?}");
        assert!(
            pair[1].2 <= pair[0].2 * 2 + 4096,
            "reflow work: {scaling:?}"
        );
    }
}

#[test]
#[ignore = "requires LICHTUNG_UNICODE16_SOURCE_DIR with hash-verified original Unicode 16 sources"]
fn contracted_line_break_analysis_matches_every_official_case_and_expanded_ascii_runs() {
    let fixture = crate::font::unicode16_source("auxiliary/LineBreakTest.txt");
    let mut cases = 0;
    for (row, source) in fixture.lines().enumerate() {
        for expansion in [1, 5] {
            let mut text = String::new();
            let mut expected = Vec::new();
            for token in source.split_whitespace() {
                match token {
                    "\u{f7}" => expected.push(text.len()),
                    "\u{d7}" => {}
                    cp => {
                        let scalar = char::from_u32(u32::from_str_radix(cp, 16).unwrap()).unwrap();
                        for _ in 0..if scalar.is_ascii_alphabetic() {
                            expansion
                        } else {
                            1
                        } {
                            text.push(scalar);
                        }
                    }
                }
            }
            let bytes = canonical::OperationFontResources::new();
            let work = canonical::OperationGenerationResources::new();
            let mut claim = |count, steps| {
                bytes.claim_bytes(count)?;
                claim_text_generation(work.claim_work(u128::from(steps)))
            };
            let actual = paragraph_linebreaks(&text, &mut claim).unwrap();
            assert_eq!(
                actual.iter().map(|&(offset, _)| offset).collect::<Vec<_>>(),
                expected,
                "row {} expansion {expansion}: {source}",
                row + 1
            );
            let reference = crate::layout::unicode16::linebreaks(&text, &mut claim).unwrap();
            assert_eq!(actual, reference, "row {} expansion {expansion}", row + 1);
        }
        cases += 1;
    }
    assert_eq!(cases, 16_672);
}

#[test]
fn contracted_line_break_analysis_has_independent_ascii_expectations() {
    use crate::layout::unicode16::BreakOpportunity::{Allowed, Mandatory};
    for (text, expected) in [
        ("Alphabet", vec![(8, Mandatory)]),
        ("Alpha Beta", vec![(6, Allowed), (10, Mandatory)]),
        ("Alpha-Beta", vec![(6, Allowed), (10, Mandatory)]),
        ("Alpha\r\nBeta", vec![(7, Mandatory), (11, Mandatory)]),
        ("Alpha\u{0308}Beta", vec![(11, Mandatory)]),
        ("Alpha\u{200B}Beta", vec![(8, Allowed), (12, Mandatory)]),
        ("Alpha\u{00A0}Beta", vec![(11, Mandatory)]),
        (
            "Alpha 1}2 Beta",
            vec![(6, Allowed), (8, Allowed), (10, Allowed), (14, Mandatory)],
        ),
    ] {
        let bytes = canonical::OperationFontResources::new();
        let work = canonical::OperationGenerationResources::new();
        let actual = paragraph_linebreaks(text, &mut |count, steps| {
            bytes.claim_bytes(count)?;
            claim_text_generation(work.claim_work(u128::from(steps)))
        })
        .unwrap();
        assert_eq!(actual, expected, "{text:?}");
    }
}

#[test]
fn reflow_reuses_analysis_through_unwrapped_wrapped_and_blank_paragraphs() {
    let environment = PreparedFontEnvironment::synthetic_faces(&[
        ("Base", &['A', ' ', '-']),
        ("Complete", &['A', ' ', '\u{301}']),
    ]);
    let mut request = tests::input(
        "AA\u{301} A\u{ad}AA\n\nA A A",
        i64::MAX,
        PhysicalTextAlignment::Start,
    );
    request.stack = "Base, Complete";
    let original = environment.plan_text(request.clone()).unwrap();
    let mut previous = original.clone();
    for width in [20_000, 14_848, i64::MAX] {
        request.maximum_width_quanta = width;
        PARAGRAPH_BIDI_CALLS.with(|calls| calls.set(0));
        let reflowed = previous.reflow(request.clone()).unwrap();
        PARAGRAPH_BIDI_CALLS.with(|calls| assert_eq!(calls.get(), 0));
        assert!(original.shares_analysis(&reflowed));
        assert!(!previous.shares_payload(&reflowed));
        assert_eq!(reflowed, environment.plan_text(request.clone()).unwrap());
        previous = reflowed;
    }
    for preferred in 1..=3 {
        canonical::FONT_SHAPE_CALLS.with(|calls| calls.set(0));
        PARAGRAPH_BIDI_CALLS.with(|calls| calls.set(0));
        let cached = original.intrinsic_from_analysis(preferred).unwrap();
        PARAGRAPH_BIDI_CALLS.with(|calls| assert_eq!(calls.get(), 0));
        if preferred > 1 {
            canonical::FONT_SHAPE_CALLS.with(|calls| assert_eq!(calls.get(), 0));
        }
        let fresh = environment
            .text_intrinsic_requirements(request.clone(), preferred)
            .unwrap();
        assert_eq!(
            (
                cached.hard_width_quanta,
                cached.natural_width_quanta,
                cached.preferred_width_quanta
            ),
            (
                fresh.hard_width_quanta,
                fresh.natural_width_quanta,
                fresh.preferred_width_quanta
            )
        );
    }
    request.font_size += 1;
    assert!(original.reflow(request).is_err());
}

#[test]
fn review_selected_soft_hyphen_paints_and_participates_in_fit() {
    let environment = PreparedFontEnvironment::synthetic_faces(&[("Soft", &['A', '-'])]);
    let mut request = tests::input("A\u{ad}AA", 12_288, PhysicalTextAlignment::Start);
    request.stack = "Soft";
    let plan = environment.plan_text(request.clone()).unwrap();
    assert_eq!(plan.line_texts(), ["A\u{ad}", "AA"]);
    assert_eq!(plan.lines[0].logical_advance_quanta, 12_288);
    assert_eq!(
        plan.lines[0].selected_soft_hyphen,
        Some(TextSourceSpan::new(1, 3).unwrap())
    );
    assert!(plan.lines[0]
        .runs
        .iter()
        .flat_map(|r| &r.glyphs)
        .any(|g| g.cluster == 1 && g.design_x_advance > 0));
    request.maximum_width_quanta = 6_144;
    assert!(matches!(
        environment.plan_text(request),
        Err(TextPlanError::Fit(_))
    ));
    let mut request = tests::input("A\u{ad}AA", i64::MAX, PhysicalTextAlignment::Start);
    request.stack = "Soft";
    let whole = environment.plan_text(request).unwrap();
    assert_eq!(whole.source, "A\u{ad}AA");
    assert_eq!(whole.lines[0].selected_soft_hyphen, None);
    assert_eq!(whole.lines[0].logical_advance_quanta, 18_432);
}

#[test]
fn selected_soft_hyphen_uses_the_sealed_literal_hyphen_fallback() {
    let environment =
        PreparedFontEnvironment::synthetic_faces(&[("Base", &['A']), ("Hyphen", &['-'])]);
    let stack = environment.parse_family_stack("Base, Hyphen").unwrap();
    environment
        .prepare_pair(
            0,
            &stack,
            RequestedWeight::Normal,
            &['A', '\u{ad}', '-'],
            None,
        )
        .unwrap();
    environment
        .prepare_source_units(0, "A\u{ad}AA", None)
        .unwrap();
    environment.seal_prepared_pairs().unwrap();
    let mut request = tests::input("A\u{ad}AA", 12_288, PhysicalTextAlignment::Start);
    request.stack = "Base, Hyphen";
    let planned = environment.plan_prepared_text(request).unwrap();
    assert_eq!(planned.source, "A\u{ad}AA");
    let runs = &planned.lines[0].runs;
    assert_eq!(runs.len(), 2);
    assert_ne!(runs[0].face, runs[1].face);
    assert_eq!(runs[1].source_span, TextSourceSpan::new(1, 3).unwrap());
    assert_eq!(runs[1].glyphs[0].cluster, 1);
    assert_eq!(runs[1].glyphs[0].glyph_origin_x_quanta, 6_144);
    assert_eq!(runs[1].design_x_advance, 600);
    assert_eq!(planned.into_line_texts(&environment).unwrap(), ["A-", "AA"]);
}

#[test]
fn selected_soft_hyphen_preserves_same_face_gpos_and_original_clusters() {
    let fonts = contextual_fonts()
        .into_iter()
        .map(|bytes| {
            let raw = ttf_parser::RawFace::parse(&bytes, 0).unwrap();
            let tables = raw
                .table_records
                .into_iter()
                .map(|record| {
                    let tag = record.tag.0.to_be_bytes();
                    let mut data = raw.table(record.tag).unwrap().to_vec();
                    if tag == *b"cmap" {
                        // Reuse glyph 8 for '-' instead of space.
                        data[28..32].copy_from_slice(&0x2d_u32.to_be_bytes());
                        data[32..36].copy_from_slice(&0x2d_u32.to_be_bytes());
                    } else if tag == *b"GPOS" {
                        // Pair positioning A + hyphen reduces A's advance by 100.
                        data = layout_table(&[(
                            *b"kern",
                            2,
                            words(&[1, 12, 4, 0, 1, 18, 1, 1, 1, 1, 8, 0xff9c]),
                        )]);
                    }
                    (tag, data)
                })
                .collect();
            sfnt(tables)
        })
        .collect();
    let environment = PreparedFontEnvironment::synthetic_shaping_fonts(fonts);
    let mut request = tests::input("A\u{ad}A", 11_264, PhysicalTextAlignment::Start);
    request.stack = "First";
    let plan = environment.plan_text(request.clone()).unwrap();
    let mut wide_request = request.clone();
    wide_request.maximum_width_quanta = i64::MAX;
    let wide = environment.plan_text(wide_request).unwrap();
    let reflowed = wide.reflow(request).unwrap();
    assert_eq!(reflowed, plan);
    assert!(wide.shares_analysis(&reflowed));
    assert_eq!(plan.source, "A\u{ad}A");
    assert_eq!(plan.lines[0].runs.len(), 1);
    let run = &plan.lines[0].runs[0];
    assert_eq!(run.source_span, TextSourceSpan::new(0, 3).unwrap());
    assert_eq!(
        run.glyphs
            .iter()
            .map(|g| (g.glyph_id, g.cluster, g.design_x_advance))
            .collect::<Vec<_>>(),
        [(1, 0, 500), (8, 1, 600)]
    );
    assert_eq!(plan.lines[0].logical_advance_quanta, 11_264);
    let direct = environment
        .shape_run(run.face, "A-", run.direction, rustybuzz::script::LATIN)
        .unwrap();
    assert_eq!(
        direct
            .glyphs
            .iter()
            .map(|g| (g.glyph_id, g.x_advance, g.x_offset, g.y_offset))
            .collect::<Vec<_>>(),
        run.glyphs
            .iter()
            .map(|g| (
                g.glyph_id,
                g.design_x_advance,
                g.design_x_offset,
                g.design_y_offset
            ))
            .collect::<Vec<_>>()
    );
}

#[test]
fn owned_line_extraction_uses_the_same_root_byte_ledger() {
    let environment = PreparedFontEnvironment::synthetic();
    let plan = environment
        .plan_text(tests::input("A", i64::MAX, PhysicalTextAlignment::Start))
        .unwrap();
    let used = environment.resource_usage().unwrap().font_bytes;
    environment
        .claim_unicode_bytes(u64::try_from(536_870_912 - used).unwrap())
        .unwrap();
    assert!(matches!(
        plan.into_line_texts(&environment),
        Err(TextPlanError::Font(FontError::Limit {
            resource: "system-font bytes",
            ..
        }))
    ));
}

#[test]
fn review_context_survives_forced_face_boundaries() {
    let environment = contextual_environment();
    let source = "\u{628}\u{628}\u{64e}\u{628}";
    let mut request = tests::input(source, i64::MAX, PhysicalTextAlignment::Start);
    request.stack = "First, Second";
    let plan = environment.plan_text(request).unwrap();
    assert_eq!(plan.source, source);
    let runs = &plan.lines[0].runs;
    assert_eq!(runs.len(), 3);
    assert_eq!(runs[0].face, runs[2].face);
    assert_ne!(runs[0].face, runs[1].face);
    assert_ne!(runs[0].units_per_em, runs[1].units_per_em);
    let whole = environment
        .shape_run(
            runs[1].face,
            source,
            ShapingDirection::RightToLeft,
            rustybuzz::script::ARABIC,
        )
        .unwrap();
    assert_eq!(
        whole.glyphs.iter().map(|g| g.glyph_id).collect::<Vec<_>>(),
        [7, 2, 6, 5]
    );
    assert_eq!(
        runs.iter()
            .flat_map(|r| &r.glyphs)
            .map(|g| (
                g.glyph_id,
                g.design_x_advance,
                g.design_x_offset,
                g.design_y_offset
            ))
            .collect::<Vec<_>>(),
        whole
            .glyphs
            .iter()
            .map(|g| (g.glyph_id, g.x_advance, g.x_offset, g.y_offset))
            .collect::<Vec<_>>()
    );
    // A selected line ending breaks joining even when more paragraph text follows.
    let isolated = environment
        .shape_run_with_context(
            runs[2].face,
            "\u{628}",
            ShapingDirection::RightToLeft,
            rustybuzz::script::ARABIC,
            canonical::ShapingContext::default(),
        )
        .unwrap();
    assert_eq!(isolated.glyphs[0].glyph_id, 3);
}

#[test]
fn wrapped_arabic_context_stops_at_selected_line_boundaries() {
    let environment = contextual_environment();
    let source = "\u{628}\u{628}\u{64e}\u{ad}\u{628}\u{628}";
    let mut request = tests::input(source, 15_360, PhysicalTextAlignment::Start);
    request.stack = "First, Second";
    let plan = environment.plan_text(request.clone()).unwrap();
    let mut wide_request = request.clone();
    wide_request.maximum_width_quanta = i64::MAX;
    let wide = environment.plan_text(wide_request).unwrap();
    let reflowed = wide.reflow(request).unwrap();
    assert_eq!(reflowed, plan);
    assert!(wide.shares_analysis(&reflowed));
    assert_eq!(plan.source, source);
    assert_eq!(plan.lines.len(), 2);
    assert_eq!(
        plan.lines[0].selected_soft_hyphen,
        Some(TextSourceSpan::new(6, 8).unwrap())
    );
    assert_eq!(plan.lines[0].logical_advance_quanta, 15_360);
    assert_eq!(
        plan.lines[0]
            .runs
            .iter()
            .flat_map(|r| &r.glyphs)
            .map(|g| g.glyph_id)
            .collect::<Vec<_>>(),
        [8, 2, 7, 5]
    );
    assert_eq!(
        plan.lines[1]
            .runs
            .iter()
            .flat_map(|r| &r.glyphs)
            .map(|g| (g.glyph_id, g.cluster))
            .collect::<Vec<_>>(),
        [(7, 10), (5, 8)]
    );
}

#[test]
fn review_canonical_output_admission_precedes_retention() {
    let environment = PreparedFontEnvironment::synthetic();
    let identity = tests::plan("A", i64::MAX, PhysicalTextAlignment::Start).lines[0].runs[0].face;
    environment.claim_text_generation_slots(1_000_000).unwrap();
    canonical::SHAPED_GLYPH_RETAINS.with(|count| count.set(0));
    assert!(environment
        .shape_run(
            identity,
            "A",
            ShapingDirection::LeftToRight,
            rustybuzz::script::LATIN
        )
        .is_err());
    canonical::SHAPED_GLYPH_RETAINS.with(|count| assert_eq!(count.get(), 0));
}

#[test]
fn review_unresolved_script_extensions_do_not_pick_an_arbitrary_script() {
    let environment = PreparedFontEnvironment::synthetic();
    let identity = tests::plan("A", i64::MAX, PhysicalTextAlignment::Start).lines[0].runs[0].face;
    for source in ["\u{30fc}", "\u{301}", "A\u{30fc}\u{628}"] {
        let items = paragraph_items(&environment, source, |_, _| Ok(identity)).unwrap();
        let index = usize::from(source.starts_with('A'));
        assert_eq!(items[index].script, Script::Common, "{source:?}");
    }
}

#[test]
fn review_hard_metrics_round_outward_for_empty_lines() {
    let metrics = DesignMetrics {
        units_per_em: 2048,
        ascender: 801,
        descender: -201,
    };
    assert_eq!(scaled_metrics(metrics, 1).unwrap(), (401, -101));
    let line = empty_line(TextSourceSpan::new(0, 0).unwrap(), 0, metrics, 1).unwrap();
    assert_eq!((line.ascent, line.descent), (401, 101));
    let environment = PreparedFontEnvironment::synthetic();
    let stack = environment.parse_family_stack("Test Sans").unwrap();
    let pair = environment
        .resolve_pair(0, &stack, RequestedWeight::Normal, &[' '])
        .unwrap();
    let mut shaped = environment
        .shape_run(
            pair.primary,
            " ",
            ShapingDirection::LeftToRight,
            rustybuzz::script::COMMON,
        )
        .unwrap();
    shaped.metrics = metrics;
    let scaled = scale_run(
        &environment,
        shaped,
        TextSourceSpan::new(1, 2).unwrap(),
        Script::Common,
        1,
        ExactRational::ZERO,
        ExactRational::ZERO,
        0,
    )
    .unwrap();
    assert_eq!(scaled.plan.logical_advance_quanta, 125);
    assert_eq!(
        (scaled.plan.ascender_quanta, scaled.plan.descender_quanta),
        (401, -101)
    );
    let mut whitespace = empty_line(TextSourceSpan::new(1, 2).unwrap(), 1, metrics, 1).unwrap();
    whitespace.advance = scaled.advance_x;
    whitespace.plan.logical_advance_quanta = scaled.plan.logical_advance_quanta;
    whitespace.plan.runs.push(scaled.plan);
    let mut request = tests::input("\n ", i64::MAX, PhysicalTextAlignment::Start);
    request.font_size = 1;
    let plan = finalize_plan(
        &environment,
        request,
        stack,
        pair.provenance,
        1,
        vec![line, whitespace],
    )
    .unwrap();
    assert_eq!(plan.line_pitch_quanta, 502);
    assert_eq!(plan.height_quanta, 1004);
    assert_eq!(plan.first_baseline_quanta, Some(401));
    assert_eq!(plan.last_baseline_quanta, Some(903));
}

#[test]
fn actual_gsub_count_and_root_bytes_gate_every_glyph_copy() {
    let environment = contextual_environment();
    let stack = environment.parse_family_stack("Second").unwrap();
    let pair = environment
        .resolve_pair(0, &stack, RequestedWeight::Normal, &['A', '\u{301}'])
        .unwrap();
    environment.claim_text_generation_slots(999_999).unwrap();
    canonical::SHAPED_GLYPH_RETAINS.with(|count| count.set(0));
    let shaped = environment
        .shape_run(
            pair.primary,
            "A\u{301}",
            ShapingDirection::LeftToRight,
            rustybuzz::script::LATIN,
        )
        .unwrap();
    assert_eq!(
        shaped.glyphs.len(),
        1,
        "admit actual ligature output, not input scalar count"
    );
    canonical::SHAPED_GLYPH_RETAINS.with(|count| assert_eq!(count.get(), 1));
    assert!(matches!(
        scale_run(
            &environment,
            shaped,
            TextSourceSpan::new(0, 3).unwrap(),
            Script::Latin,
            1,
            ExactRational::ZERO,
            ExactRational::ZERO,
            0
        ),
        Err(TextPlanError::Limit {
            resource: "candidate-generation slots",
            ..
        })
    ));

    let environment = PreparedFontEnvironment::synthetic();
    let stack = environment.parse_family_stack("Test Sans").unwrap();
    let pair = environment
        .resolve_pair(0, &stack, RequestedWeight::Normal, &['A'])
        .unwrap();
    let used = environment.resource_usage().unwrap().font_bytes;
    environment
        .claim_unicode_bytes(u64::try_from(536_870_912 - used).unwrap())
        .unwrap();
    canonical::SHAPED_GLYPH_RETAINS.with(|count| count.set(0));
    assert!(matches!(
        environment.shape_run(
            pair.primary,
            "A",
            ShapingDirection::LeftToRight,
            rustybuzz::script::LATIN
        ),
        Err(FontError::Limit {
            resource: "system-font bytes",
            ..
        })
    ));
    canonical::SHAPED_GLYPH_RETAINS.with(|count| assert_eq!(count.get(), 0));
    let request = tests::input("", 0, PhysicalTextAlignment::Start);
    assert!(matches!(
        finalize_plan(&environment, request, stack, pair.provenance, 1, vec![]),
        Err(TextPlanError::Font(FontError::Limit {
            resource: "system-font bytes",
            ..
        }))
    ));
}

#[test]
fn source_units_select_complete_faces_without_reassigning_standalone_bases() {
    let environment = PreparedFontEnvironment::synthetic_faces(&[
        ("Base", &['A']),
        ("Acute", &['A', '\u{301}']),
        ("Grave", &['A', '\u{300}']),
    ]);
    let source = "AA\u{301}A\nA\u{300}AA\u{301}";
    let mut request = tests::input(source, i64::MAX, PhysicalTextAlignment::Start);
    request.stack = "Base, Acute, Grave";
    let planned = environment.plan_text(request.clone()).unwrap();
    let stack = environment.parse_family_stack(request.stack).unwrap();
    let pair = environment
        .prepared_pair(&stack, RequestedWeight::Normal)
        .unwrap()
        .unwrap();
    let [base, acute, grave] = pair.provenance.candidates.as_slice() else {
        panic!("three real SFNT candidates");
    };
    assert_eq!(planned.source, source);
    assert_eq!(planned.line_texts(), ["AA\u{301}A", "A\u{300}AA\u{301}"]);
    for (line, expected) in planned.lines.iter().zip([
        vec![("A", base), ("A\u{301}", acute), ("A", base)],
        vec![("A\u{300}", grave), ("A", base), ("A\u{301}", acute)],
    ]) {
        assert_eq!(line.runs.len(), expected.len());
        for (run, (text, identity)) in line.runs.iter().zip(expected) {
            assert_eq!(&source[run.source_span.range()], text);
            assert_eq!(run.face, *identity);
            let whole = environment
                .shape_run(
                    *identity,
                    text,
                    ShapingDirection::LeftToRight,
                    rustybuzz::script::LATIN,
                )
                .unwrap();
            assert_eq!(
                run.glyphs
                    .iter()
                    .map(|glyph| (
                        glyph.glyph_id,
                        glyph.design_x_advance,
                        glyph.design_y_advance,
                        glyph.design_x_offset,
                        glyph.design_y_offset
                    ))
                    .collect::<Vec<_>>(),
                whole
                    .glyphs
                    .iter()
                    .map(|glyph| (
                        glyph.glyph_id,
                        glyph.x_advance,
                        glyph.y_advance,
                        glyph.x_offset,
                        glyph.y_offset
                    ))
                    .collect::<Vec<_>>()
            );
        }
    }
    // Reacquisition must retain the units prepared by the unsealed query.
    assert_eq!(pair.selected_units.len(), 2);
    environment.seal_prepared_pairs().unwrap();
    canonical::FONT_PAIR_RESOLUTION_CALLS.with(|calls| calls.set(0));
    assert_eq!(
        environment.plan_prepared_text(request.clone()).unwrap(),
        planned
    );
    request.source = "AA\u{301}\u{301}";
    for prepared_only in [false, true] {
        let error = if prepared_only {
            environment.plan_prepared_text(request.clone())
        } else {
            environment.plan_text(request.clone())
        }
        .unwrap_err();
        assert!(matches!(
            error,
            TextPlanError::Font(FontError::UnknownUnit { pair_ordinal: 0 })
        ));
    }
    assert_eq!(
        environment
            .prepared_pair(&stack, RequestedWeight::Normal)
            .unwrap()
            .unwrap(),
        pair
    );
    canonical::FONT_PAIR_RESOLUTION_CALLS.with(|calls| assert_eq!(calls.get(), 0));
}

#[test]
fn unsealed_queries_extend_scalars_and_prepare_new_units_before_selection() {
    let environment = PreparedFontEnvironment::synthetic_faces(&[
        ("Base", &['A']),
        ("Complete", &['A', '\u{301}']),
    ]);
    let mut request = tests::input("A", i64::MAX, PhysicalTextAlignment::Start);
    request.stack = "Base, Complete";
    let standalone = environment.plan_text(request.clone()).unwrap();
    request.source = "AA\u{301}A";
    let marked = environment.plan_text(request.clone()).unwrap();
    let runs = &marked.lines[0].runs;
    assert_eq!(runs.len(), 3);
    assert_eq!(runs[0].face, standalone.lines[0].runs[0].face);
    assert_ne!(runs[1].face, runs[0].face);
    assert_eq!(runs[2].face, runs[0].face);
    let intrinsic = environment
        .text_intrinsic_requirements(request.clone(), 1)
        .unwrap();
    assert_eq!(intrinsic.natural_width_quanta, marked.width_quanta);
    assert_eq!(environment.plan_text(request.clone()).unwrap(), marked);
    environment.seal_prepared_pairs().unwrap();
    assert_eq!(environment.plan_prepared_text(request).unwrap(), marked);
}

#[test]
fn uncovered_whole_units_use_the_canonical_scalar_fallback() {
    let environment =
        PreparedFontEnvironment::synthetic_faces(&[("Base", &['A']), ("Mark", &['\u{301}'])]);
    let mut request = tests::input("A\u{301}A", i64::MAX, PhysicalTextAlignment::Start);
    request.stack = "Base, Mark";
    let planned = environment.plan_text(request).unwrap();
    let runs = &planned.lines[0].runs;
    assert_eq!(runs.len(), 3);
    assert_eq!(runs[0].face, runs[2].face);
    assert_ne!(runs[0].face, runs[1].face);
    assert_eq!(&planned.source[runs[1].source_span.range()], "\u{301}");
    assert!(runs.iter().all(|run| run.script == Script::Latin));
    let stack = environment.parse_family_stack("Base, Mark").unwrap();
    let pair = environment
        .prepared_pair(&stack, RequestedWeight::Normal)
        .unwrap()
        .unwrap();
    assert_eq!(
        pair.selected_units,
        [canonical::SelectedFontUnit {
            source: "A\u{301}".into(),
            face: None
        }]
    );
}

#[test]
fn fallback_controls_keep_source_and_neighbor_face_without_nominal_coverage() {
    let environment = PreparedFontEnvironment::synthetic_faces(&[
        ("Base", &['A']),
        ("Complete", &['A', '\u{301}']),
    ]);
    let source = "\u{200d}A\u{301}\u{fe0f}\u{200c}A";
    let mut request = tests::input(source, i64::MAX, PhysicalTextAlignment::Start);
    request.stack = "Base, Complete";
    let planned = environment.plan_text(request).unwrap();
    assert_eq!(planned.source, source);
    let runs = &planned.lines[0].runs;
    assert_eq!(runs.len(), 2);
    assert_eq!(
        &source[runs[0].source_span.range()],
        "\u{200d}A\u{301}\u{fe0f}\u{200c}"
    );
    assert_eq!(&source[runs[1].source_span.range()], "A");
    assert_ne!(runs[0].face, runs[1].face);
    let direct = environment
        .shape_run(
            runs[0].face,
            &source[runs[0].source_span.range()],
            ShapingDirection::LeftToRight,
            rustybuzz::script::LATIN,
        )
        .unwrap();
    assert_eq!(
        runs[0]
            .glyphs
            .iter()
            .map(|g| (
                g.glyph_id,
                g.design_x_advance,
                g.design_x_offset,
                g.design_y_offset
            ))
            .collect::<Vec<_>>(),
        direct
            .glyphs
            .iter()
            .map(|g| (g.glyph_id, g.x_advance, g.x_offset, g.y_offset))
            .collect::<Vec<_>>()
    );
}

#[test]
fn control_only_text_does_not_require_a_space_or_control_glyph() {
    let environment = PreparedFontEnvironment::synthetic_faces(&[("Base", &['A'])]);
    let source = "\u{2067}\u{200d}\u{2069}";
    let mut request = tests::input(source, i64::MAX, PhysicalTextAlignment::Start);
    request.stack = "Base";
    let planned = environment.plan_text(request).unwrap();
    assert_eq!(planned.source, source);
    assert_eq!(planned.lines[0].logical_advance_quanta, 0);
    assert_eq!(planned.lines[0].ink_quanta, None);
}

#[test]
fn fallback_runs_with_different_font_units_share_one_exact_pen_and_outward_ink() {
    let environment = PreparedFontEnvironment::synthetic_faces(&[
        ("Base", &['A']),
        ("Complete", &['A', '\u{301}']),
    ]);
    let mut request = tests::input("AA\u{301}A", i64::MAX, PhysicalTextAlignment::Start);
    request.stack = "Base, Complete";
    let planned = environment.plan_text(request).unwrap();
    let mut pen_x = ExactRational::ZERO;
    let mut pen_y = ExactRational::ZERO;
    let mut rounded_run_sum = 0;
    let mut ink: Option<ExactInk> = None;
    let mut origins = Vec::new();
    for (index, run) in planned.lines[0].runs.iter().enumerate() {
        let mut shaped = environment
            .shape_run(
                run.face,
                &planned.source[run.source_span.range()],
                run.direction,
                rustybuzz::script::LATIN,
            )
            .unwrap();
        shaped.metrics.units_per_em = if index == 1 { 2048 } else { 3000 };
        for glyph in &mut shaped.glyphs {
            glyph.x_advance = 1;
            glyph.y_advance = if index == 1 { -1 } else { 0 };
            glyph.x_offset = if index == 1 { 1 } else { -1 };
            glyph.y_offset = -glyph.x_offset;
            glyph.ink = Some(GlyphInk {
                x_min: -1,
                y_min: -1,
                x_max: 1,
                y_max: 1,
            });
        }
        let built = scale_run(
            &environment,
            shaped,
            run.source_span,
            run.script,
            1,
            pen_x,
            pen_y,
            run.bidi_level,
        )
        .unwrap();
        pen_x = pen_x.add(built.advance_x, "test line advance").unwrap();
        pen_y = pen_y.add(built.advance_y, "test line advance").unwrap();
        rounded_run_sum += built.plan.logical_advance_quanta;
        if let Some(bounds) = built.ink {
            ink = Some(ink.map_or(bounds, |prior| prior.union(bounds).unwrap()));
        }
        origins.extend(
            built
                .plan
                .glyphs
                .iter()
                .map(|g| (g.glyph_origin_x_quanta, g.glyph_origin_y_quanta)),
        );
    }
    assert_eq!(origins, [(0, 0), (1, 0), (1, -1), (1, -1)]);
    assert_eq!(pen_x, ExactRational::new(631, 375).unwrap());
    assert_eq!(pen_x.ceil_i64("wrap").unwrap(), 2);
    assert_eq!(pen_x.nearest_even_i64("line").unwrap(), 2);
    assert_eq!(
        rounded_run_sum, 1,
        "summing rounded run advances would drift"
    );
    assert_eq!(
        ink.unwrap().outward().unwrap(),
        TextInkBounds {
            left: -1,
            bottom: -2,
            right: 2,
            top: 1
        }
    );
}

#[test]
fn wrapped_plans_keep_paragraph_embedding_context_and_line_l1_levels() {
    let environment = PreparedFontEnvironment::synthetic();
    let source = "A \u{202b}\u{5d0} \u{5d0} \u{202c}A";
    let planned = environment
        .plan_text(tests::input(source, 8_704, PhysicalTextAlignment::Start))
        .unwrap();
    assert!(planned.lines.len() >= 3);
    assert_eq!(planned.line_texts().concat(), source);
    let paragraph = ParagraphBidiInfo::new(source, None);
    let mut differs_from_independent_line = false;
    for line in &planned.lines {
        let range = line.source_span.range();
        let (levels, visual) = paragraph.visual_runs(range.clone());
        let independent = ParagraphBidiInfo::new(&source[range.clone()], None);
        differs_from_independent_line |= independent.paragraph_level != paragraph.paragraph_level;
        assert_eq!(line.base_direction, ShapingDirection::LeftToRight);
        let mut visual_index = 0;
        for run in &line.runs {
            let start = run.source_span.start_byte as usize;
            while !visual[visual_index].contains(&start) {
                visual_index += 1;
            }
            assert_eq!(run.bidi_level, levels[start].number());
            assert_eq!(
                run.direction == ShapingDirection::RightToLeft,
                levels[start].is_rtl()
            );
            assert!(run
                .glyphs
                .iter()
                .all(|glyph| range.contains(&(glyph.cluster as usize))));
        }
    }
    assert!(
        differs_from_independent_line,
        "fixture must distinguish paragraph from line resolution"
    );
}

fn words(values: &[u16]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_be_bytes()).collect()
}

fn set_word(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
}

// One DFLT language system, with one lookup per named feature.
fn layout_table(features: &[([u8; 4], u16, Vec<u8>)]) -> Vec<u8> {
    let count = u16::try_from(features.len()).unwrap();
    let mut scripts = words(&[1]);
    scripts.extend_from_slice(b"DFLT");
    scripts.extend(words(&[8, 4, 0, 0, 0xffff, count]));
    scripts.extend(words(&(0..count).collect::<Vec<_>>()));
    let mut feature_list = words(&[count]);
    for (index, (tag, _, _)) in features.iter().enumerate() {
        feature_list.extend(tag);
        feature_list.extend(words(&[2 + count * 6 + u16::try_from(index).unwrap() * 6]));
    }
    for index in 0..count {
        feature_list.extend(words(&[0, 1, index]));
    }
    let mut lookup_list = words(&[count]);
    let mut lookups = Vec::new();
    for (_, kind, subtable) in features {
        lookup_list.extend(words(&[2
            + count * 2
            + u16::try_from(lookups.len()).unwrap()]));
        lookups.extend(words(&[*kind, 0, 1, 8]));
        lookups.extend(subtable);
    }
    lookup_list.extend(lookups);
    let mut result = words(&[
        1,
        0,
        10,
        10 + u16::try_from(scripts.len()).unwrap(),
        10 + u16::try_from(scripts.len() + feature_list.len()).unwrap(),
    ]);
    result.extend(scripts);
    result.extend(feature_list);
    result.extend(lookup_list);
    result
}

/// No installed fonts or binary assets: actual GSUB ccmp/init/medi/fina and
/// GPOS single positioning, with distinct glyphs for every contextual form.
fn contextual_font() -> Vec<u8> {
    let mut head = vec![0; 54];
    set_word(&mut head, 18, 1000);
    let mut hhea = vec![0; 36];
    set_word(&mut hhea, 4, 800);
    set_word(&mut hhea, 6, 0xff38);
    set_word(&mut hhea, 34, 9);
    let maxp = words(&[0, 0x5000, 9]);
    let hmtx = words(&[600, 0].repeat(9));
    let mut cmap = words(&[0, 1, 3, 10]);
    cmap.extend(12_u32.to_be_bytes());
    cmap.extend(words(&[12, 0]));
    cmap.extend(76_u32.to_be_bytes());
    cmap.extend(0_u32.to_be_bytes());
    cmap.extend(5_u32.to_be_bytes());
    for (scalar, glyph) in [
        (0x20_u32, 8_u32),
        (0x41, 1),
        (0x301, 2),
        (0x628, 3),
        (0x64e, 2),
    ] {
        cmap.extend(scalar.to_be_bytes());
        cmap.extend(scalar.to_be_bytes());
        cmap.extend(glyph.to_be_bytes());
    }
    // Ligature A + acute -> 4. Coverage starts at byte 18.
    let ligature = words(&[1, 18, 1, 8, 1, 4, 4, 2, 2, 1, 1, 1]);
    let single = |replacement| words(&[2, 8, 1, replacement, 1, 1, 3]);
    let gsub = layout_table(&[
        (*b"ccmp", 4, ligature),
        (*b"fina", 1, single(7)),
        (*b"init", 1, single(5)),
        (*b"medi", 1, single(6)),
    ]);
    // Positioned ligature: x=-17, y=23, advance adjustment=-11.
    let gpos = layout_table(&[(*b"kern", 1, words(&[1, 12, 7, 0xffef, 23, 0xfff5, 1, 1, 4]))]);
    let tables = vec![
        (*b"head", head),
        (*b"hhea", hhea),
        (*b"maxp", maxp),
        (*b"hmtx", hmtx),
        (*b"cmap", cmap),
        (*b"GSUB", gsub),
        (*b"GPOS", gpos),
    ];
    sfnt(tables)
}

fn sfnt(mut tables: Vec<([u8; 4], Vec<u8>)>) -> Vec<u8> {
    tables.sort_by_key(|(tag, _)| *tag);
    let mut font = words(&[1, 0, u16::try_from(tables.len()).unwrap(), 0, 0, 0]);
    let mut offset = 12 + tables.len() * 16;
    for (tag, data) in &tables {
        font.extend(tag);
        font.extend(0_u32.to_be_bytes());
        font.extend(u32::try_from(offset).unwrap().to_be_bytes());
        font.extend(u32::try_from(data.len()).unwrap().to_be_bytes());
        offset += data.len().next_multiple_of(4);
    }
    for (_, data) in tables {
        font.extend(data);
        font.resize(font.len().next_multiple_of(4), 0);
    }
    font
}

fn contextual_environment() -> PreparedFontEnvironment {
    PreparedFontEnvironment::synthetic_shaping_fonts(contextual_fonts())
}

fn contextual_fonts() -> Vec<Vec<u8>> {
    let names = PreparedFontEnvironment::synthetic_faces(&[("First", &['A']), ("Second", &['A'])]);
    let mut database = fontdb::Database::new();
    names.populate_database(&mut database).unwrap();
    let context = contextual_font();
    let context = ttf_parser::RawFace::parse(&context, 0).unwrap();
    database
        .faces()
        .map(|face| {
            let second = face.families.iter().any(|(name, _)| name == "Second");
            database
                .with_face_data(face.id, |bytes, index| {
                    let raw = ttf_parser::RawFace::parse(bytes, index).unwrap();
                    let mut tables: Vec<_> = raw
                        .table_records
                        .into_iter()
                        .map(|record| {
                            (
                                record.tag.0.to_be_bytes(),
                                raw.table(record.tag).unwrap().to_vec(),
                            )
                        })
                        .collect();
                    for tag in [*b"GSUB", *b"GPOS", *b"cmap", *b"maxp", *b"hmtx"] {
                        tables.retain(|(old, _)| *old != tag);
                        tables.push((
                            tag,
                            context
                                .table(ttf_parser::Tag::from_bytes(&tag))
                                .unwrap()
                                .to_vec(),
                        ));
                    }
                    for (tag, data) in &mut tables {
                        match &*tag {
                            b"head" if second => set_word(data, 18, 2000),
                            b"cmap" if !second => {
                                // Remove the fatha group, rather than mapping it to .notdef.
                                data.truncate(76);
                                data[16..20].copy_from_slice(&64_u32.to_be_bytes());
                                data[24..28].copy_from_slice(&4_u32.to_be_bytes());
                            }
                            b"loca" => data.extend(words(&[17; 6])),
                            _ => {}
                        }
                        if *tag == *b"cmap" {
                            data[28..32].copy_from_slice(&0x2d_u32.to_be_bytes());
                            data[32..36].copy_from_slice(&0x2d_u32.to_be_bytes());
                        }
                    }
                    sfnt(tables)
                })
                .unwrap()
        })
        .collect()
}

fn shape(
    face: &rustybuzz::Face<'_>,
    source: &str,
    script: Script,
    direction: ShapingDirection,
) -> Vec<(u32, i32, i32, i32, i32)> {
    let mut buffer = rustybuzz::UnicodeBuffer::new();
    buffer.push_str(source);
    buffer.set_script(rustybuzz_script(script).unwrap());
    buffer.set_direction(direction.into());
    let result = rustybuzz::shape(face, &[], buffer);
    result
        .glyph_infos()
        .iter()
        .zip(result.glyph_positions())
        .map(|(info, pos)| {
            (
                info.glyph_id,
                pos.x_advance,
                pos.y_advance,
                pos.x_offset,
                pos.y_offset,
            )
        })
        .collect()
}

#[test]
fn generated_gsub_gpos_preserve_base_accent_and_arabic_context() {
    let bytes = contextual_font();
    let face = rustybuzz::Face::from_slice(&bytes, 0).unwrap();
    let environment = PreparedFontEnvironment::synthetic();
    let identity = tests::plan("A", i64::MAX, PhysicalTextAlignment::Start).lines[0].runs[0].face;
    for (source, script, direction, expected) in [
        (
            "A\u{301}",
            Script::Latin,
            ShapingDirection::LeftToRight,
            vec![(4, 589, 0, -17, 23)],
        ),
        (
            "\u{628}\u{64e}\u{628}\u{628}",
            Script::Arabic,
            ShapingDirection::RightToLeft,
            vec![
                (7, 600, 0, 0, 0),
                (6, 600, 0, 0, 0),
                (2, 0, 0, 0, 0),
                (5, 600, 0, 0, 0),
            ],
        ),
    ] {
        let whole = shape(&face, source, script, direction);
        assert_eq!(whole, expected);
        let items = paragraph_items(&environment, source, |_, _| Ok(identity)).unwrap();
        let groups = itemize_bidi_run(&environment, &items, 0, 0..source.len()).unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].script, script);
        let planned = shape(
            &face,
            &source[groups[0].span.range()],
            groups[0].script,
            direction,
        );
        assert_eq!(planned, whole);
        let separate: Vec<_> = source
            .chars()
            .flat_map(|c| shape(&face, &c.to_string(), script, direction))
            .collect();
        assert_ne!(separate, whole, "fixture must detect lost context");
    }
}

#[test]
fn script_extensions_brackets_and_leading_marks_use_paragraph_context() {
    let environment = PreparedFontEnvironment::synthetic();
    let identity = tests::plan("A", i64::MAX, PhysicalTextAlignment::Start).lines[0].runs[0].face;
    for (source, expected) in [
        ("\u{301}A\u{301}", vec![Script::Latin; 3]),
        ("\u{628}\u{64e}\u{628}", vec![Script::Arabic; 3]),
        ("\u{30ab}\u{30fc}", vec![Script::Katakana; 2]),
        (
            "A(\u{628})\u{301} ",
            vec![
                Script::Latin,
                Script::Latin,
                Script::Arabic,
                Script::Latin,
                Script::Latin,
                Script::Latin,
            ],
        ),
        (
            "A(\u{628})A",
            vec![
                Script::Latin,
                Script::Latin,
                Script::Arabic,
                Script::Latin,
                Script::Latin,
            ],
        ),
        ("\u{3008}\u{30ab}\u{3009}", vec![Script::Katakana; 3]),
    ] {
        let items = paragraph_items(&environment, source, |_, _| Ok(identity)).unwrap();
        assert_eq!(
            items.iter().map(|i| i.script).collect::<Vec<_>>(),
            expected,
            "{source:?}"
        );
    }
}

#[test]
fn selected_line_l1_l2_equal_paragraph_reference_with_embeddings_and_isolates() {
    for source in [
        "A \u{202b}\u{5d0} 12  \u{202c} Z",
        "\u{5d0} \u{2066}A 12 \u{2069}  B",
        "A (\u{5d0} 12)  Z",
    ] {
        let bidi = ParagraphBidiInfo::new(source, None);
        let retained = ParagraphBidiData::from(ParagraphBidiInfo::new(source, None));
        let boundaries: Vec<_> = source
            .char_indices()
            .map(|(i, _)| i)
            .chain([source.len()])
            .collect();
        for (index, &start) in boundaries.iter().enumerate() {
            for &end in &boundaries[index + 1..] {
                let (expected_levels, expected_runs) = bidi.visual_runs(start..end);
                let (levels, runs) = line_visual_runs(&retained, source, start..end).unwrap();
                assert_eq!(levels, expected_levels[start..end]);
                assert_eq!(
                    runs.iter()
                        .map(|r| r.start + start..r.end + start)
                        .collect::<Vec<_>>(),
                    expected_runs
                );
            }
        }
    }
}

#[test]
fn origins_round_exact_cumulative_positions_once_and_ink_uses_those_origins() {
    let environment = PreparedFontEnvironment::synthetic();
    let identity = tests::plan("A", i64::MAX, PhysicalTextAlignment::Start).lines[0].runs[0].face;
    let mut shaped = environment
        .shape_run(
            identity,
            "AAAA",
            ShapingDirection::LeftToRight,
            rustybuzz::script::LATIN,
        )
        .unwrap();
    shaped.metrics.units_per_em = 2048;
    for glyph in &mut shaped.glyphs {
        glyph.x_advance = 1;
        glyph.y_advance = -1;
        glyph.x_offset = 1;
        glyph.y_offset = -1;
        glyph.ink = Some(GlyphInk {
            x_min: 0,
            y_min: 0,
            x_max: 1,
            y_max: 1,
        });
    }
    let run = scale_run(
        &environment,
        shaped,
        TextSourceSpan::new(0, 4).unwrap(),
        Script::Latin,
        1,
        ExactRational::new(1, 2).unwrap(),
        ExactRational::new(-1, 2).unwrap(),
        0,
    )
    .unwrap();
    assert_eq!(
        run.plan
            .glyphs
            .iter()
            .map(|g| (g.glyph_origin_x_quanta, g.glyph_origin_y_quanta))
            .collect::<Vec<_>>(),
        [(1, -1), (2, -2), (2, -2), (2, -2)]
    );
    assert_eq!(run.advance_x, ExactRational::new(2, 1).unwrap());
    assert_eq!(
        run.plan.ink_quanta,
        Some(TextInkBounds {
            left: 1,
            bottom: -2,
            right: 3,
            top: 0
        })
    );
    assert_eq!(
        run.plan.glyphs[1].ink_quanta,
        Some(TextInkBounds {
            left: 2,
            bottom: -2,
            right: 3,
            top: -1
        })
    );
}
