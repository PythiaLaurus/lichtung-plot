use super::super::admission::{ScanReason, TableState};
use super::*;

fn retained(bytes: Vec<u8>) -> RetainedFontFile {
    RetainedFontFile {
        root_ordinal: 0,
        canonical_path: PathBuf::from("generated-admission.ttf"),
        sha256: Sha256::digest(&bytes).into(),
        bytes: Arc::new(bytes),
    }
}

fn core(fonts: Vec<Vec<u8>>) -> CanonicalFontCore {
    CanonicalFontCore::from_retained(
        fonts.into_iter().map(retained).collect(),
        &HostGenericNames::default(),
        FontPreparation::new(),
        scalar_presence_map(),
    )
    .unwrap()
}

fn tables(bytes: &[u8]) -> Vec<([u8; 4], Vec<u8>)> {
    let raw = ttf_parser::RawFace::parse(bytes, 0).unwrap();
    raw.table_records
        .into_iter()
        .map(|record| {
            (
                record.tag.0.to_be_bytes(),
                raw.table(record.tag).unwrap().to_vec(),
            )
        })
        .collect()
}

#[test]
fn family_without_id6_and_after_bad_record_is_admitted() {
    let mut data = tables(&build_test_font());
    let name = &mut data.iter_mut().find(|(tag, _)| tag == b"name").unwrap().1;
    // First record is out of bounds; ID16 is later and valid; ID6 is absent.
    set_u16(name, 6 + 10, u16::MAX);
    set_u16(name, 6 + 2 * 12 + 6, 2);
    let core = core(vec![build_sfnt(data)]);
    assert_eq!(core.faces().len(), 1);
    assert_eq!(core.faces()[0].names.len(), 1);
    assert_eq!(core.faces()[0].names[0].record_ordinal, 1);
    assert!(core
        .database
        .face(core.faces()[0].database_id)
        .unwrap()
        .post_script_name
        .is_empty());
    assert!(
        core.fonts.issues.is_empty(),
        "unknown record and absent optional ID6 are not scan issues"
    );
    let environment = PreparedFontEnvironment::for_core(core).unwrap();
    let pair = environment
        .resolve_pair(
            0,
            &named_stack(&["Test Sans"]),
            RequestedWeight::Normal,
            &['A'],
        )
        .unwrap();
    assert_eq!(pair.primary.face_index, 0);
    assert!(!environment
        .shape_run(
            pair.primary,
            "A",
            ShapingDirection::LeftToRight,
            rustybuzz::script::LATIN
        )
        .unwrap()
        .glyphs
        .is_empty());
}

#[test]
fn unrelated_bad_or_unsupported_names_do_not_add_scan_requirements() {
    let mut data = tables(&build_test_font());
    let name = &mut data.iter_mut().find(|(tag, _)| tag == b"name").unwrap().1;
    // Valid ID16 remains. A malformed Unicode copyright and a non-Unicode
    // FullName are neither matching keys nor the optional ID6 mirror.
    set_u16(name, 6 + 6, 0);
    set_u16(name, 42, 0xd800);
    set_u16(name, 6 + 2 * 12, 1); // Macintosh, not the supported Unicode encoding.
    set_u16(name, 6 + 2 * 12 + 2, 0);
    set_u16(name, 6 + 2 * 12 + 6, 4);
    let bytes = build_sfnt(data);
    let parsed = ttf_parser::Face::parse(&bytes, 0).unwrap();
    assert!(parsed.names().get(0).unwrap().to_string().is_none());
    assert!(!parsed.names().get(2).unwrap().is_unicode());
    let core = core(vec![bytes]);
    assert!(
        core.fonts.issues.is_empty(),
        "no diagnostics allocated for unconsumed names/defaults"
    );
    assert_eq!(core.faces()[0].names.len(), 1);
    assert_eq!(core.faces()[0].names[0].value, "Test Sans");
    assert_eq!(core.faces()[0].names[0].record_ordinal, 1);
    let mirror = core.database.face(core.faces()[0].database_id).unwrap();
    assert_eq!(mirror.families[0].0, "Test Sans");
    assert!(mirror.post_script_name.is_empty());
    let environment = PreparedFontEnvironment::for_core(core).unwrap();
    let pair = environment
        .resolve_pair(
            0,
            &named_stack(&["Test Sans"]),
            RequestedWeight::Normal,
            &['A'],
        )
        .unwrap();
    assert!(environment
        .shape_run(
            pair.primary,
            "A",
            ShapingDirection::LeftToRight,
            rustybuzz::script::LATIN
        )
        .unwrap()
        .glyphs[0]
        .ink
        .is_some());
}

#[test]
fn rejected_content_keeps_bytes_and_precise_classification() {
    let all = tables(&build_test_font());
    let name_only = all
        .iter()
        .filter(|(tag, _)| tag == b"name")
        .cloned()
        .collect();
    let mut apple = vec![
        (*b"bhed", vec![0; 54]),
        (*b"bdat", vec![0; 4]),
        (*b"bloc", vec![0; 4]),
    ];
    apple.extend(all.iter().filter(|(tag, _)| tag == b"name").cloned());
    let mut bad_head = all.clone();
    bad_head
        .iter_mut()
        .find(|(tag, _)| tag == b"head")
        .unwrap()
        .1
        .truncate(8);
    let mut empty = all;
    empty.retain(|(tag, _)| !matches!(tag, b"glyf" | b"loca"));
    let core = core(vec![
        build_test_font(),
        build_sfnt(name_only),
        build_sfnt(apple),
        build_sfnt(bad_head),
        build_sfnt(empty),
    ]);
    assert_eq!(core.files().len(), 5);
    assert_eq!(core.faces().len(), 2, "all-empty glyphs are not image-only");
    let parser_issues: Vec<_> = core
        .fonts
        .issues
        .iter()
        .filter(|issue| matches!(issue.reason, ScanReason::Parser { .. }))
        .collect();
    assert_eq!(parser_issues.len(), 3);
    for (issue, ordinal, apple_bitmap_tables, head) in [
        (parser_issues[0], 1, false, TableState::Absent),
        (parser_issues[1], 2, true, TableState::Absent),
        (parser_issues[2], 3, false, TableState::Readable),
    ] {
        assert_eq!(issue.file_ordinal, ordinal);
        assert_eq!(issue.face_index, Some(0));
        assert_eq!(
            issue.reason,
            ScanReason::Parser {
                error: ttf_parser::FaceParsingError::NoHeadTable,
                head,
                apple_bitmap_tables,
            }
        );
        assert_eq!(
            core.files()[ordinal].sha256,
            Sha256::digest(core.files()[ordinal].bytes.as_slice()).as_slice()
        );
    }
}

#[test]
fn collection_sibling_failure_preserves_original_index_and_truncated_count_is_bounded() {
    let sfnt = build_test_font();
    let mut collection = b"ttcf\0\x01\0\0\0\0\0\x02".to_vec();
    push_u32(&mut collection, u32::MAX);
    push_u32(&mut collection, 20);
    let mut shifted = sfnt.clone();
    for ordinal in 0..usize::from(u16::from_be_bytes([sfnt[4], sfnt[5]])) {
        let offset = 12 + ordinal * 16 + 8;
        let value = u32::from_be_bytes(sfnt[offset..offset + 4].try_into().unwrap());
        set_u32(&mut shifted, offset, value + 20);
    }
    collection.extend(shifted);
    let truncated = b"ttcf\0\x01\0\0\xff\xff\xff\xff".to_vec();
    let core = core(vec![collection, truncated]);
    assert_eq!(core.faces().len(), 1);
    assert_eq!(core.faces()[0].identity.face_index, 1);
    assert_eq!(
        core.database
            .face(core.faces()[0].database_id)
            .unwrap()
            .index,
        1
    );
    assert!(core
        .fonts
        .issues
        .iter()
        .any(|issue| issue.face_index == Some(0) && issue.file_ordinal == 0));
    assert!(
        core.fonts
            .issues
            .iter()
            .any(|issue| issue.face_index.is_none()
                && issue.reason == ScanReason::TruncatedCollection)
    );
    assert_eq!(
        core.fonts.faces.capacity(),
        1,
        "never reserve the untrusted TTC count"
    );
}

#[test]
fn admission_does_not_absorb_work_resource_or_internal_failures() {
    let mut storage = Vec::<FontScanIssue>::new();
    assert!(matches!(
        reserve_prepared_or_local(&mut storage, usize::MAX, "font scan diagnostics", None),
        Err(FontError::Allocation(_))
    ));
    assert!(matches!(
        CanonicalFontCore::from_retained(
            vec![retained(vec![0; 16])],
            &HostGenericNames::default(),
            FontPreparation::new(),
            Vec::new(),
        ),
        Err(FontError::Arithmetic("Unicode scalar presence map"))
    ));
}

#[cfg(unix)]
#[test]
fn a_discovered_font_target_io_error_is_not_a_smaller_admitted_catalog() {
    let directory = tempfile::tempdir().unwrap();
    fs::write(directory.path().join("good.ttf"), build_test_font()).unwrap();
    std::os::unix::fs::symlink(
        directory.path().join("gone.ttf"),
        directory.path().join("broken.ttf"),
    )
    .unwrap();
    let roots = ordinal_roots(vec![directory.path().to_path_buf()], None).unwrap();
    assert!(matches!(
        discover_retained_files(&roots, None),
        Err(FontError::Io { .. })
    ));
}

#[test]
fn image_only_declarations_and_optional_defaults_do_not_become_an_outline_sanitizer() {
    let mut image = tables(&build_test_font());
    image.retain(|(tag, _)| !matches!(tag, b"glyf" | b"loca"));
    // A declaration-only classification witness, not a claim of valid sbix data.
    image.push((*b"sbix", vec![0; 8]));
    let mut mixed = tables(&build_test_font());
    mixed.push((*b"sbix", vec![0; 8]));
    let mut optional = tables(&build_test_font());
    optional.retain(|(tag, _)| tag != b"OS/2");
    optional.push((*b"post", vec![0; 4]));
    let core = core(vec![
        build_sfnt(image),
        build_sfnt(mixed),
        build_sfnt(optional),
    ]);
    assert_eq!(
        core.faces().len(),
        2,
        "an optional paint parser None does not reject vectors"
    );
    assert_eq!(
        core.fonts.issues[0].reason,
        ScanReason::UnsupportedImageOnly
    );
    assert_eq!(core.fonts.issues[0].file_ordinal, 0);
    let optional = &core.faces()[1];
    assert_eq!(
        (optional.style, optional.stretch, optional.weight),
        (Style::Normal, Stretch::Normal, 400)
    );
    let mirror = core.database.face(optional.database_id).unwrap();
    assert!(
        !mirror.monospaced,
        "short post is not positive monospaced/italic evidence"
    );
    assert!(
        !core
            .fonts
            .issues
            .iter()
            .any(|issue| issue.file_ordinal == 2),
        "normal optional-field defaults are not invalid-font diagnostics"
    );
}

#[test]
fn required_table_ranges_and_absent_family_have_distinct_local_diagnostics() {
    let mut out_of_bounds = build_test_font();
    let directory = (12..12
        + usize::from(u16::from_be_bytes([out_of_bounds[4], out_of_bounds[5]])) * 16)
        .step_by(16)
        .find(|offset| &out_of_bounds[*offset..*offset + 4] == b"head")
        .unwrap();
    set_u32(&mut out_of_bounds, directory + 8, u32::MAX);
    let mut unaddressable = tables(&build_test_font());
    let names = &mut unaddressable
        .iter_mut()
        .find(|(tag, _)| tag == b"name")
        .unwrap()
        .1;
    set_u16(names, 6 + 6, 2);
    set_u16(names, 6 + 12 + 6, 2);
    let core = core(vec![
        out_of_bounds,
        build_sfnt(unaddressable),
        build_test_font(),
    ]);
    assert_eq!(core.faces().len(), 1);
    assert!(core.fonts.issues.iter().any(|issue| matches!(
        issue.reason,
        ScanReason::Parser {
            head: TableState::OutOfBounds,
            apple_bitmap_tables: false,
            ..
        }
    )));
    assert!(core
        .fonts
        .issues
        .iter()
        .any(|issue| issue.file_ordinal == 1 && issue.reason == ScanReason::MissingFamily));
}

#[test]
fn admission_order_is_not_candidate_order_and_failures_retain_context() {
    let one = build_test_font_with_coverage("Same", &['A']);
    let two = build_test_font_with_coverage("Same", &['A', 'B']);
    let rejected = build_sfnt(
        tables(&one)
            .into_iter()
            .filter(|(tag, _)| tag == b"name")
            .collect(),
    );
    let stack = named_stack(&["Same"]);
    let mut orders = Vec::new();
    for fonts in [
        vec![one.clone(), rejected.clone(), two.clone()],
        vec![two, one, rejected],
    ] {
        let environment = PreparedFontEnvironment::for_core(core(fonts)).unwrap();
        environment
            .admit_text(&stack, RequestedWeight::Normal, "A", None)
            .unwrap();
        orders.push(
            environment
                .prepared_pair(&stack, RequestedWeight::Normal)
                .unwrap()
                .unwrap()
                .provenance
                .candidates,
        );
        let error = environment
            .admit_text(
                &named_stack(&["Not installed"]),
                RequestedWeight::Normal,
                "A",
                None,
            )
            .unwrap_err();
        assert!(matches!(error, FontError::Admission { .. }));
        let message = error.to_string();
        for expected in [
            "Not installed",
            "generated-admission.ttf",
            "sha256",
            "NoHeadTable",
            "index=Some(0)",
        ] {
            assert!(message.contains(expected), "{message}");
        }
    }
    assert_eq!(orders[0], orders[1]);
}

// Run real host discovery in child processes: never mutate process-global font
// configuration in the parallel test harness. Only Linux has this fontconfig path.
#[cfg(target_os = "linux")]
#[test]
fn linux_public_api_ignores_unrelated_name_only_and_freezes_output() {
    const CHILD: &str = "LICHTUNG_FONT_ADMISSION_CHILD";
    if let Some(output) = std::env::var_os(CHILD) {
        let output = PathBuf::from(output);
        let canvas = crate::Canvas::grid(1, 1)
            .unwrap()
            .text_panel(
                0,
                0,
                crate::TextPanel::new("Font admission", "Actual Canvas text").unwrap(),
            )
            .unwrap()
            .size(640, 320);
        let frozen = canvas.svg(output.join("first.svg")).unwrap();
        // No reread/reselection during SVG/PNG replay after original files vanish.
        fs::remove_file(output.join("fonts/normal.ttf")).unwrap();
        frozen
            .svg(output.join("replay.svg"))
            .unwrap()
            .png(output.join("scene.png"))
            .unwrap();
        assert_eq!(
            fs::read(output.join("first.svg")).unwrap(),
            fs::read(output.join("replay.svg")).unwrap()
        );
        assert!(fs::metadata(output.join("scene.png")).unwrap().len() > 100);
        return;
    }
    let directory = tempfile::tempdir().unwrap();
    for mixed in [false, true] {
        let output = directory
            .path()
            .join(if mixed { "mixed" } else { "normal" });
        fs::create_dir_all(output.join("fonts")).unwrap();
        fs::copy(
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            output.join("fonts/normal.ttf"),
        )
        .unwrap();
        if mixed {
            let name = tables(&build_test_font())
                .into_iter()
                .filter(|(tag, _)| tag == b"name")
                .collect();
            fs::write(output.join("fonts/unrelated.ttf"), build_sfnt(name)).unwrap();
        }
        let config = output.join("fonts.conf");
        fs::write(&config, format!("<fontconfig><dir>{}</dir><alias><family>sans-serif</family><prefer><family>DejaVu Sans</family></prefer></alias><alias><family>serif</family><prefer><family>DejaVu Sans</family></prefer></alias><alias><family>monospace</family><prefer><family>DejaVu Sans</family></prefer></alias></fontconfig>", output.join("fonts").display())).unwrap();
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "font::canonical::tests::admission::linux_public_api_ignores_unrelated_name_only_and_freezes_output", "--nocapture"])
            .env(CHILD, &output).env("FONTCONFIG_FILE", config).status().unwrap();
        assert!(status.success(), "mixed={mixed}");
    }
    assert_eq!(
        fs::read(directory.path().join("normal/first.svg")).unwrap(),
        fs::read(directory.path().join("mixed/first.svg")).unwrap()
    );
}
