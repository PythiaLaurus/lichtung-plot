use super::*;

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
    let stack = canonical::named_stack(&["First"]);
    request.stack = &stack;
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
fn source_units_select_complete_faces_without_reassigning_standalone_bases() {
    let environment = PreparedFontEnvironment::synthetic_faces(&[
        ("Base", &['A']),
        ("Acute", &['A', '\u{301}']),
        ("Grave", &['A', '\u{300}']),
    ]);
    let source = "AA\u{301}A\nA\u{300}AA\u{301}";
    let mut request = tests::input(source, i64::MAX, PhysicalTextAlignment::Start);
    let stack = canonical::named_stack(&["Base", "Acute", "Grave"]);
    request.stack = &stack;
    let planned = environment.plan_text(request.clone()).unwrap();
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
    environment.operation().seal_fonts().unwrap();
    canonical::FONT_PAIR_RESOLUTION_CALLS.with(|calls| calls.set(0));
    assert_eq!(environment.plan_text(request.clone()).unwrap(), planned);
    request.source = "AA\u{301}\u{301}";
    let error = environment.plan_text(request.clone()).unwrap_err();
    assert!(matches!(
        error,
        TextPlanError::Font(FontError::PreparedPairAfterSeal { pair_ordinal: 0 })
    ));
    assert_eq!(
        environment
            .prepared_pair(&stack, RequestedWeight::Normal)
            .unwrap()
            .unwrap(),
        pair
    );
    canonical::FONT_PAIR_RESOLUTION_CALLS.with(|calls| assert_eq!(calls.get(), 0));
}

fn words(values: &[u16]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_be_bytes()).collect()
}

fn set_word(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
}

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

pub(super) fn sfnt(mut tables: Vec<([u8; 4], Vec<u8>)>) -> Vec<u8> {
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

pub(super) fn contextual_fonts() -> Vec<Vec<u8>> {
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
        shaped,
        TextSourceSpan::new(0, 4).unwrap(),
        Script::Latin,
        1.0,
        &ExactRational::new(1, 2).unwrap(),
        &ExactRational::new(-1, 2).unwrap(),
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
