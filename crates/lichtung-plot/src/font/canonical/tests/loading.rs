//! Bounded generated-input witnesses of the real file-backed B2 lifecycle.
use super::*;

pub(super) fn reset_observations() {
    LOAD_OBSERVATIONS.with_borrow_mut(|observed| *observed = LoadObservations::default());
}

pub(super) fn observations() -> LoadObservations {
    LOAD_OBSERVATIONS.with_borrow(Clone::clone)
}

pub(super) fn print_observations(label: &str, observed: &LoadObservations) {
    eprintln!(
        "{label}: metadata_bytes={} full_bytes={} hash_bytes={} materializations={} capability_faces={} frozen_bytes={} frozen_files={}",
        observed.metadata_bytes.values().sum::<usize>(),
        observed.full_bytes,
        observed.hash_bytes,
        observed.materialized.len(),
        observed.face_checks.len(),
        observed.frozen_files.iter().map(|(_, length)| length).sum::<usize>(),
        observed.frozen_files.len(),
    );
    eprintln!("{observed:#?}");
}

pub(super) fn assert_two_operations(first: &LoadObservations, total: &LoadObservations) {
    assert!(!first.metadata_bytes.is_empty());
    assert!(first.full_bytes > 0);
    assert_eq!(total.full_bytes, first.full_bytes * 2);
    assert_eq!(total.hash_bytes, first.hash_bytes * 2);
    assert_eq!(first.hash_bytes, first.full_bytes);
    assert_eq!(total.metadata_bytes.len(), first.metadata_bytes.len());
    for (path, count) in &first.metadata_bytes {
        assert_eq!(total.metadata_bytes[path], count * 2);
    }
    assert_eq!(
        total.materialized,
        [&first.materialized[..], &first.materialized[..]].concat()
    );
    assert_eq!(
        total.face_checks,
        [&first.face_checks[..], &first.face_checks[..]].concat()
    );
    assert_eq!(total.frozen_files, first.frozen_files);
}

fn indexed_environment(root: &Path, generic: &str) -> PreparedFontEnvironment {
    let roots = ordinal_roots(vec![root.to_path_buf()], None).unwrap();
    let index = discover_indexed_files(&roots, None).unwrap();
    PreparedFontEnvironment::for_core(
        CanonicalFontCore::from_index(
            index,
            &HostGenericNames {
                sans_serif: Some(generic.into()),
                ..HostGenericNames::default()
            },
            FontPreparation::new(),
            scalar_presence_map(),
        )
        .unwrap(),
    )
    .unwrap()
}

pub(super) fn name_only(family: &str, payload: usize) -> Vec<u8> {
    let font = build_test_font_with_coverage(family, &['A']);
    let raw = ttf_parser::RawFace::parse(&font, 0).unwrap();
    build_sfnt(vec![
        (
            *b"name",
            raw.table(ttf_parser::Tag::from_bytes(b"name"))
                .unwrap()
                .to_vec(),
        ),
        (*b"zzzz", vec![0x56; payload]),
    ])
}

fn collection(faces: &[Option<Vec<u8>>]) -> Vec<u8> {
    let mut bytes = b"ttcf\0\x01\0\0".to_vec();
    push_u32(&mut bytes, u32::try_from(faces.len()).unwrap());
    for _ in faces {
        push_u32(&mut bytes, u32::MAX);
    }
    for (index, face) in faces.iter().enumerate() {
        let Some(face) = face else { continue };
        let offset = u32::try_from(bytes.len()).unwrap();
        set_u32(&mut bytes, 12 + index * 4, offset);
        let mut face = face.clone();
        for record in 0..usize::from(u16::from_be_bytes([face[4], face[5]])) {
            let position = 12 + record * 16 + 8;
            let old = u32::from_be_bytes(face[position..position + 4].try_into().unwrap());
            set_u32(&mut face, position, old + offset);
        }
        bytes.extend(face);
    }
    bytes
}

#[test]
fn candidate_union_not_unrelated_payload_controls_full_read_hash_and_retention() {
    let good = build_test_font_with_coverage("Used", &['A', 'B']);
    let alternate = build_test_font_with_coverage("Used", &['A']);
    let bad = name_only("Used", 0);
    let mut results = Vec::new();
    for payload in [64, 2 * 1024 * 1024] {
        let directory = tempfile::tempdir().unwrap();
        fs::write(directory.path().join("good.ttf"), &good).unwrap();
        fs::write(directory.path().join("alternate.ttf"), &alternate).unwrap();
        fs::write(directory.path().join("matched-bad.ttf"), &bad).unwrap();
        let unrelated = name_only("Unused", payload);
        assert!(ttf_parser::Face::parse(&unrelated, 0).is_err());
        fs::write(directory.path().join("unrelated.ttf"), unrelated).unwrap();
        fs::write(directory.path().join("unaddressable.ttf"), [0; 16]).unwrap();
        reset_observations();
        let environment = indexed_environment(directory.path(), "Used");
        assert_eq!(observations().full_bytes, 0);
        assert!(observations().face_checks.is_empty());
        assert!(environment.core.read().unwrap().fonts.files.is_empty());
        let stack = named_stack(&["Used"]);
        environment
            .admit_text(&stack, RequestedWeight::Normal, "A", None)
            .unwrap();
        let first = environment
            .prepared_pair(&stack, RequestedWeight::Normal)
            .unwrap()
            .unwrap();
        let mut expected = [&good, &alternate].map(|bytes| FaceIdentity {
            sha256: Sha256::digest(bytes).into(),
            face_index: 0,
        });
        expected.sort();
        assert_eq!(first.provenance.candidates, expected);
        {
            let core = environment.core.read().unwrap();
            assert_eq!(core.fonts.files.len(), 3);
            assert!(core.fonts.issues.iter().any(|issue| matches!(
                issue.reason,
                super::super::admission::ScanReason::Parser {
                    error: ttf_parser::FaceParsingError::NoHeadTable,
                    ..
                }
            )));
        }
        let loaded = observations();
        assert_eq!(loaded.full_bytes, good.len() + alternate.len() + bad.len());
        assert_eq!(loaded.hash_bytes, loaded.full_bytes);
        assert_eq!(loaded.materialized.len(), 3);
        assert_eq!(loaded.face_checks.len(), 3);
        assert!(loaded
            .materialized
            .iter()
            .all(|(path, _)| path.file_name().unwrap() != "unrelated.ttf"));
        assert!(loaded.metadata_bytes[&directory.path().join("unrelated.ttf")] < payload + 256);

        // New weight, generic pair and new source share the already closed file
        // union. None may invalidate or replace the original scalar choice.
        environment
            .admit_text(&stack, RequestedWeight::Bold, "B", None)
            .unwrap();
        environment
            .admit_text(&stack, RequestedWeight::Normal, "B", None)
            .unwrap();
        let generic = FontStack::new([FontFamily::generic(GenericFontFamily::SansSerif)]).unwrap();
        environment
            .admit_text(&generic, RequestedWeight::Normal, "A", None)
            .unwrap();
        assert_eq!(observations(), loaded);
        let extended = environment
            .prepared_pair(&stack, RequestedWeight::Normal)
            .unwrap()
            .unwrap();
        assert_eq!(extended.provenance, first.provenance);
        assert_eq!(extended.selected_scalars[0], first.selected_scalars[0]);
        environment.operation().unwrap().seal_fonts().unwrap();
        let frozen = SealedTextVerifier::new(&environment)
            .retain_fonts()
            .unwrap();
        let retained = frozen
            .files
            .iter()
            .map(|file| file.bytes.len())
            .sum::<usize>();
        assert_eq!(retained, good.len() + alternate.len() + bad.len());
        results.push((
            loaded.metadata_bytes.values().sum::<usize>(),
            loaded.full_bytes,
            retained,
        ));
        eprintln!(
            "generated payload={payload} metadata={} full={} hash={} retained={retained}",
            results.last().unwrap().0,
            loaded.full_bytes,
            loaded.hash_bytes
        );
    }
    assert_eq!(results[0], results[1]);
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one shared TTC witness exercises late admission, fixed generics, seal and visitation order"
)]
fn collection_indices_late_pairs_fixed_generic_and_seal_have_one_materialization_boundary() {
    let directory = tempfile::tempdir().unwrap();
    let ttc = collection(&[
        None,
        Some(build_test_font_with_coverage("One", &['A', 'B'])),
        Some(build_test_font_with_coverage("Two", &['A'])),
    ]);
    let later = build_test_font_with_coverage("Later", &['A']);
    fs::write(directory.path().join("shared.ttc"), &ttc).unwrap();
    fs::write(directory.path().join("later.ttf"), &later).unwrap();
    fs::write(
        directory.path().join("never.ttf"),
        build_test_font_with_coverage("Never", &['A']),
    )
    .unwrap();
    fs::write(
        directory.path().join("truncated.ttc"),
        b"ttcf\0\x01\0\0\xff\xff\xff\xff",
    )
    .unwrap();
    reset_observations();
    let environment = indexed_environment(directory.path(), "One");
    let one = named_stack(&["One"]);
    let two = named_stack(&["Two"]);
    environment
        .admit_text(&one, RequestedWeight::Normal, "A", None)
        .unwrap();
    let original = environment
        .prepared_pair(&one, RequestedWeight::Normal)
        .unwrap()
        .unwrap();
    assert_eq!(original.primary.face_index, 1);
    assert_eq!(
        original.primary.sha256.as_slice(),
        Sha256::digest(&ttc).as_slice()
    );
    let first = observations();
    assert_eq!(first.full_bytes, ttc.len());
    assert_eq!(
        first
            .face_checks
            .iter()
            .map(|(_, index)| *index)
            .collect::<Vec<_>>(),
        [0, 1, 2]
    );
    environment
        .admit_text(&two, RequestedWeight::Bold, "A", None)
        .unwrap();
    assert_eq!(
        environment
            .prepared_pair(&two, RequestedWeight::Bold)
            .unwrap()
            .unwrap()
            .primary
            .face_index,
        2
    );
    assert_eq!(observations(), first);
    environment
        .admit_text(&named_stack(&["Later"]), RequestedWeight::Normal, "A", None)
        .unwrap();
    assert_eq!(observations().full_bytes, ttc.len() + later.len());
    let generic = FontStack::new([FontFamily::generic(GenericFontFamily::SansSerif)]).unwrap();
    environment
        .admit_text(&generic, RequestedWeight::Normal, "A", None)
        .unwrap();
    assert_eq!(
        environment
            .prepared_pair(&generic, RequestedWeight::Normal)
            .unwrap()
            .unwrap()
            .primary,
        original.primary
    );
    assert_eq!(
        environment
            .prepared_pair(&one, RequestedWeight::Normal)
            .unwrap()
            .unwrap(),
        original
    );
    assert_eq!(observations().materialized.len(), 2);
    environment.operation().unwrap().seal_fonts().unwrap();
    let before = observations();
    assert!(environment
        .admit_text(&named_stack(&["Never"]), RequestedWeight::Normal, "A", None)
        .is_err());
    assert!(environment
        .admit_text(&one, RequestedWeight::Normal, "B", None)
        .is_err());
    assert_eq!(observations(), before);
    let registry = SealedTextVerifier::new(&environment)
        .retain_fonts()
        .unwrap();
    assert_eq!(registry.files.len(), 2);
    assert_eq!(
        registry
            .files
            .iter()
            .map(|file| file.bytes.len())
            .sum::<usize>(),
        ttc.len() + later.len()
    );

    // Different file/pair visitation order changes storage ordinals, not SHA
    // ranking or selections. First-use pair ordinals deliberately differ.
    let reordered = indexed_environment(directory.path(), "One");
    reordered
        .admit_text(&named_stack(&["Later"]), RequestedWeight::Normal, "A", None)
        .unwrap();
    reordered
        .admit_text(&two, RequestedWeight::Bold, "A", None)
        .unwrap();
    reordered
        .admit_text(&one, RequestedWeight::Normal, "A", None)
        .unwrap();
    let selected = reordered
        .prepared_pair(&one, RequestedWeight::Normal)
        .unwrap()
        .unwrap();
    assert_eq!(
        selected.provenance.candidates,
        original.provenance.candidates
    );
    assert_eq!(selected.primary, original.primary);
    assert_eq!(selected.selected_scalars, original.selected_scalars);
}

#[test]
fn consumed_metadata_changes_fail_even_with_restored_mtime_and_do_not_publish_a_pair() {
    let foo = build_test_font_with_coverage("Foo", &['A']);
    let mut directory_change = foo.clone();
    directory_change[16] ^= 1; // A directory checksum, not a changed family string.
    for changed in [
        build_test_font_with_coverage("Bar", &['A']),
        directory_change,
    ] {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("font.ttf");
        fs::write(&path, &foo).unwrap();
        let environment = indexed_environment(directory.path(), "Foo");
        let old_modified = fs::metadata(&path).unwrap().modified().unwrap();
        assert_eq!(foo.len(), changed.len());
        fs::write(&path, &changed).unwrap();
        File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_times(fs::FileTimes::new().set_modified(old_modified))
            .unwrap();
        assert_eq!(
            fs::metadata(&path).unwrap().modified().unwrap(),
            old_modified
        );
        let stack = named_stack(&["Foo"]);
        reset_observations();
        assert!(matches!(
            environment.admit_text(&stack, RequestedWeight::Normal, "A", None),
            Err(FontError::ChangedFile(changed)) if changed == path
        ));
        assert_eq!(observations().full_bytes, changed.len());
        assert!(observations().face_checks.is_empty());
        assert!(environment
            .prepared_pair(&stack, RequestedWeight::Normal)
            .unwrap()
            .is_none());
    }
}

#[test]
fn indexed_source_replacement_and_materialization_io_are_not_content_rejections() {
    for replacement in [true, false] {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("font.ttf");
        let bytes = build_test_font_with_coverage("Foo", &['A']);
        fs::write(&path, &bytes).unwrap();
        let environment = indexed_environment(directory.path(), "Foo");
        if replacement {
            fs::rename(&path, directory.path().join("original-kept-for-identity")).unwrap();
            fs::write(&path, &bytes).unwrap();
        } else {
            fs::remove_file(&path).unwrap();
        }
        let result =
            environment.admit_text(&named_stack(&["Foo"]), RequestedWeight::Normal, "A", None);
        if replacement {
            assert!(matches!(result, Err(FontError::ChangedFile(_))));
        } else {
            assert!(matches!(result, Err(FontError::Io { .. })));
        }
        assert!(environment.core.read().unwrap().prepared_pairs.is_empty());
    }
}

#[test]
fn unread_payload_has_no_past_snapshot_before_first_materialization() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("font.ttf");
    let mut tables = super::admission::tables(&build_test_font_with_coverage("Foo", &['A']));
    tables.push((*b"zzzz", vec![0x56; 4096]));
    let original = build_sfnt(tables);
    fs::write(&path, &original).unwrap();
    let environment = indexed_environment(directory.path(), "Foo");
    let modified = fs::metadata(&path).unwrap().modified().unwrap();
    let mut changed = original.clone();
    *changed.last_mut().unwrap() ^= 1; // Not a consumed metadata range.
    fs::write(&path, &changed).unwrap();
    File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_times(fs::FileTimes::new().set_modified(modified))
        .unwrap();
    let stack = named_stack(&["Foo"]);
    environment
        .admit_text(&stack, RequestedWeight::Normal, "A", None)
        .unwrap();
    let pair = environment
        .prepared_pair(&stack, RequestedWeight::Normal)
        .unwrap()
        .unwrap();
    assert_ne!(Sha256::digest(&original), Sha256::digest(&changed));
    assert_eq!(
        pair.primary.sha256.as_slice(),
        Sha256::digest(&changed).as_slice()
    );
    environment.operation().unwrap().seal_fonts().unwrap();
    let registry = SealedTextVerifier::new(&environment)
        .retain_fonts()
        .unwrap();
    assert_eq!(registry.files[0].bytes.as_slice(), changed);
    fs::remove_file(path).unwrap();
    assert!(environment
        .shape_run(
            pair.primary,
            "A",
            ShapingDirection::LeftToRight,
            rustybuzz::script::LATIN
        )
        .is_ok());
}

// Explicit, opt-in observation of existing fixed local inputs. Never runs in
// routine gates, scans no extra roots, and never changes/deletes supplied fonts.
#[cfg(target_os = "linux")]
#[test]
#[ignore = "manual fixed FONTCONFIG_FILE resource observation, not a speed benchmark"]
fn observe_fixed_fontconfig_candidate_loading() {
    assert!(
        std::env::var_os("FONTCONFIG_FILE").is_some(),
        "supply an isolated FONTCONFIG_FILE"
    );
    let directory = tempfile::tempdir().unwrap();
    let canvas = || {
        crate::Canvas::grid(1, 1)
            .unwrap()
            .text_panel(
                0,
                0,
                crate::TextPanel::new("Font admission", "Actual Canvas text").unwrap(),
            )
            .unwrap()
            .size(640, 320)
    };
    reset_observations();
    let first = canvas().svg(directory.path().join("first.svg")).unwrap();
    let once = observations();
    let second = canvas()
        .svg_preserve_text(directory.path().join("preserved.svg"))
        .unwrap();
    let twice = observations();
    assert_two_operations(&once, &twice);
    print_observations("OPERATION_1", &once);
    print_observations("TWO_FRESH_OPERATIONS", &twice);
    second
        .svg(directory.path().join("second.svg"))
        .unwrap()
        .png_preview(directory.path().join("preview.png"))
        .unwrap();
    first.svg(directory.path().join("replay.svg")).unwrap();
    assert_eq!(observations(), twice);
    assert_eq!(
        fs::read(directory.path().join("first.svg")).unwrap(),
        fs::read(directory.path().join("second.svg")).unwrap()
    );
}
