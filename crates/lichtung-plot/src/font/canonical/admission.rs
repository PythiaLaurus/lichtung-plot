//! The only content-rejection boundary. I/O and work-resource errors never enter
//! this module as parser failures; selected faces are not retried here.
use super::*;

pub(super) fn format_diagnostic(arguments: std::fmt::Arguments<'_>) -> Result<String, FontError> {
    let mut count = CensusFormatCount::default();
    count
        .write_fmt(arguments)
        .map_err(|_| FontError::Arithmetic("font diagnostic bytes"))?;
    let mut output = String::new();
    output
        .try_reserve_exact(count.bytes)
        .map_err(|_| FontError::Allocation("font diagnostics"))?;
    output
        .write_fmt(arguments)
        .map_err(|_| FontError::Arithmetic("font diagnostic formatting"))?;
    Ok(output)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TableState {
    Absent,
    OutOfBounds,
    Readable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ScanReason {
    TruncatedCollection,
    Parser {
        error: ttf_parser::FaceParsingError,
        head: TableState,
        apple_bitmap_tables: bool,
    },
    UnsupportedImageOnly,
    MissingFamily,
    MissingPostScriptName,
    UnusableNameRecord(u16),
    OptionalDefault([u8; 4], TableState),
}

#[derive(Debug)]
pub(super) struct FontScanIssue {
    pub(super) file_ordinal: usize,
    pub(super) face_index: Option<u32>,
    pub(super) reason: ScanReason,
}

fn issue(
    issues: &mut Vec<FontScanIssue>,
    file_ordinal: usize,
    face_index: Option<u32>,
    reason: ScanReason,
    stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<(), FontError> {
    reserve_prepared_or_local(issues, 1, "font scan diagnostics", stage)?;
    issues.push(FontScanIssue {
        file_ordinal,
        face_index,
        reason,
    });
    Ok(())
}

fn table_state(raw: &ttf_parser::RawFace<'_>, tag: [u8; 4]) -> TableState {
    let tag = ttf_parser::Tag::from_bytes(&tag);
    if raw.table(tag).is_some() {
        TableState::Readable
    } else if raw
        .table_records
        .into_iter()
        .any(|record| record.tag == tag)
    {
        TableState::OutOfBounds
    } else {
        TableState::Absent
    }
}

/// Validate the offset array before any count-derived work or reservation.
/// Individual bad offsets are subsequently rejected at their original index.
fn face_count(bytes: &[u8]) -> Option<u32> {
    if bytes.get(..4) != Some(b"ttcf") {
        return Some(1);
    }
    let count = ttf_parser::fonts_in_collection(bytes)?;
    let end = usize::try_from(count)
        .ok()?
        .checked_mul(4)?
        .checked_add(12)?;
    (end <= bytes.len()).then_some(count)
}

#[expect(
    clippy::too_many_lines,
    reason = "one ordered content-admission transaction"
)]
pub(super) fn admit_file(
    file: &RetainedFontFile,
    file_ordinal: usize,
    database: &mut Database,
    faces: &mut Vec<FaceMetadata>,
    issues: &mut Vec<FontScanIssue>,
    mut stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<(), FontError> {
    let Some(count) = face_count(&file.bytes) else {
        return issue(
            issues,
            file_ordinal,
            None,
            ScanReason::TruncatedCollection,
            stage,
        );
    };
    for index in 0..count {
        let raw = ttf_parser::RawFace::parse(&file.bytes, index);
        let parsed = match ttf_parser::Face::parse(&file.bytes, index) {
            Ok(face) => face,
            Err(error) => {
                let head = raw
                    .as_ref()
                    .map_or(TableState::Absent, |raw| table_state(raw, *b"head"));
                let apple_bitmap_tables = raw.as_ref().is_ok_and(|raw| {
                    table_state(raw, *b"bhed") == TableState::Readable
                        && [b"bdat", b"bloc"]
                            .iter()
                            .all(|tag| table_state(raw, **tag) == TableState::Readable)
                });
                issue(
                    issues,
                    file_ordinal,
                    Some(index),
                    ScanReason::Parser {
                        error,
                        head,
                        apple_bitmap_tables,
                    },
                    stage.as_deref_mut(),
                )?;
                continue;
            }
        };
        let raw = parsed.raw_face();
        // Declarations, not optional parser None, distinguish image-only from
        // legitimate all-empty glyphs. This is not an all-table sanitizer.
        let vector_declared = [b"glyf", b"CFF ", b"CFF2"]
            .iter()
            .any(|tag| table_state(raw, **tag) != TableState::Absent);
        let image_declared = [b"sbix", b"CBDT", b"EBDT", b"bdat"]
            .iter()
            .any(|tag| table_state(raw, **tag) != TableState::Absent);
        if !vector_declared && image_declared {
            issue(
                issues,
                file_ordinal,
                Some(index),
                ScanReason::UnsupportedImageOnly,
                stage.as_deref_mut(),
            )?;
            continue;
        }
        for ordinal in 0..parsed.names().len() {
            if parsed
                .names()
                .get(ordinal)
                .is_none_or(|record| !decodes_like_ttf_parser(&record))
            {
                issue(
                    issues,
                    file_ordinal,
                    Some(index),
                    ScanReason::UnusableNameRecord(ordinal),
                    stage.as_deref_mut(),
                )?;
            }
        }
        let names = decode_family_names(&parsed, stage.as_deref_mut())?;
        if names.is_empty() {
            issue(
                issues,
                file_ordinal,
                Some(index),
                ScanReason::MissingFamily,
                stage.as_deref_mut(),
            )?;
            continue;
        }
        for (tag, usable) in [
            (*b"OS/2", parsed.tables().os2.is_some()),
            (
                *b"post",
                raw.table(ttf_parser::Tag::from_bytes(b"post"))
                    .is_some_and(|data| data.len() >= 16),
            ),
        ] {
            if !usable {
                issue(
                    issues,
                    file_ordinal,
                    Some(index),
                    ScanReason::OptionalDefault(tag, table_state(raw, tag)),
                    stage.as_deref_mut(),
                )?;
            }
        }
        let mut families = Vec::new();
        reserve_prepared_or_local(
            &mut families,
            names.len(),
            "fontdb family mirror",
            stage.as_deref_mut(),
        )?;
        for name in &names {
            let mut value = String::new();
            reserve_string_prepared_or_local(
                &mut value,
                name.value.len(),
                "fontdb family mirror",
                stage.as_deref_mut(),
            )?;
            value.push_str(&name.value);
            let language = parsed
                .names()
                .get(name.record_ordinal)
                .map_or(fontdb::Language::Unknown, |record| record.language());
            families.push((value, language));
        }
        let mut post_script_name = String::new();
        for ordinal in 0..parsed.names().len() {
            if let Some(record) = parsed.names().get(ordinal) {
                if record.name_id == ttf_parser::name_id::POST_SCRIPT_NAME {
                    if let Some(value) = decode_name(&record)? {
                        post_script_name = value;
                        break;
                    }
                }
            }
        }
        if post_script_name.is_empty() {
            issue(
                issues,
                file_ordinal,
                Some(index),
                ScanReason::MissingPostScriptName,
                stage.as_deref_mut(),
            )?;
        }
        let post = raw.table(ttf_parser::Tag::from_bytes(b"post"));
        let nonzero = |range| {
            post.and_then(|data| data.get(range))
                .is_some_and(|bytes| bytes != [0; 4])
        };
        let mut style = match parsed.style() {
            ttf_parser::Style::Normal => Style::Normal,
            ttf_parser::Style::Italic => Style::Italic,
            ttf_parser::Style::Oblique => Style::Oblique,
        };
        if style == Style::Normal && nonzero(4..8) {
            style = Style::Italic;
        }
        reserve_prepared_or_local(faces, 1, "font face metadata", stage.as_deref_mut())?;
        let database_id = database.push_face_info(fontdb::FaceInfo {
            id: ID::dummy(),
            source: Source::Binary(Arc::new(BinarySource(Arc::clone(&file.bytes)))),
            index,
            families,
            post_script_name,
            style,
            weight: fontdb::Weight(parsed.weight().to_number()),
            stretch: parsed.width(),
            monospaced: nonzero(12..16),
        });
        faces.push(FaceMetadata {
            identity: FaceIdentity {
                sha256: file.sha256,
                face_index: index,
            },
            file_ordinal,
            database_id,
            names,
            style,
            stretch: parsed.width(),
            weight: parsed.weight().to_number(),
            metrics: design_metrics(&parsed),
        });
    }
    Ok(())
}

/// Same Unicode decoding as the locked parser, but allocation is a work error,
/// not an absent/bad name. No extra naming, language or PostScript gate.
pub(super) fn decode_name(
    record: &ttf_parser::name::Name<'_>,
) -> Result<Option<String>, FontError> {
    if !decodes_like_ttf_parser(record) {
        return Ok(None);
    }
    let scalars = || {
        char::decode_utf16(
            record
                .name
                .chunks_exact(2)
                .map(|unit| u16::from_be_bytes([unit[0], unit[1]])),
        )
    };
    let length = scalars().try_fold(0_usize, |length, scalar| {
        length
            .checked_add(scalar.map_or(0, char::len_utf8))
            .ok_or(FontError::Arithmetic("decoded font name bytes"))
    })?;
    let mut result = String::new();
    result
        .try_reserve_exact(length)
        .map_err(|_| FontError::Allocation("decoded font name"))?;
    for scalar in scalars() {
        result.push(scalar.map_err(|_| FontError::Arithmetic("font name decoding changed"))?);
    }
    Ok(Some(result))
}
