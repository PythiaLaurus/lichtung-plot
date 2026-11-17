//! Operation-local family index. Only directories, name records and consumed
//! ID16/ID1 strings are read. No glyph payload, full-file hash or Face capability
//! parse belongs to this phase. All table/name interpretation uses the locked
//! parser; the small range transport does not repair a font.
use super::*;
use std::io::{Seek, SeekFrom};
use std::time::SystemTime;

#[derive(Debug, PartialEq, Eq)]
struct SourceStamp {
    length: u64,
    modified: Option<SystemTime>,
    created: Option<SystemTime>,
    identity: (u64, u64),
}

fn stamp(file: &File, path: &Path) -> Result<SourceStamp, FontError> {
    let metadata = file.metadata().map_err(|source| FontError::Io {
        operation: "inspect indexed font",
        path: path.to_path_buf(),
        source,
    })?;
    if !metadata.is_file() {
        return Err(FontError::ChangedFile(path.to_path_buf()));
    }
    Ok(SourceStamp {
        length: metadata.len(),
        modified: metadata.modified().ok(),
        created: metadata.created().ok(),
        identity: file_identity(file, &metadata, path)?,
    })
}

#[cfg(unix)]
#[expect(
    clippy::unnecessary_wraps,
    reason = "same fallible interface on Windows"
)]
fn file_identity(
    _file: &File,
    metadata: &fs::Metadata,
    _path: &Path,
) -> Result<(u64, u64), FontError> {
    use std::os::unix::fs::MetadataExt;
    Ok((metadata.dev(), metadata.ino()))
}

#[cfg(windows)]
#[allow(unsafe_code)]
fn file_identity(
    file: &File,
    _metadata: &fs::Metadata,
    path: &Path,
) -> Result<(u64, u64), FontError> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::{
        GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
    };
    let mut info = std::mem::MaybeUninit::<BY_HANDLE_FILE_INFORMATION>::uninit();
    // SAFETY: the borrowed File keeps this handle live; the OS writes the full
    // information structure on success, which is checked before assume_init.
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), info.as_mut_ptr()) } == 0 {
        return Err(FontError::Io {
            operation: "identify indexed font",
            path: path.to_path_buf(),
            source: std::io::Error::last_os_error(),
        });
    }
    // SAFETY: successful GetFileInformationByHandle initialized the structure.
    let info = unsafe { info.assume_init() };
    Ok((
        u64::from(info.dwVolumeSerialNumber),
        u64::from(info.nFileIndexHigh) << 32 | u64::from(info.nFileIndexLow),
    ))
}

#[cfg(not(any(unix, windows)))]
#[expect(
    clippy::unnecessary_wraps,
    reason = "same fallible interface on Windows"
)]
fn file_identity(
    _file: &File,
    _metadata: &fs::Metadata,
    _path: &Path,
) -> Result<(u64, u64), FontError> {
    Ok((0, 0))
}

#[derive(Debug)]
enum IndexedSource {
    File(SourceStamp),
    #[cfg(test)]
    Generated(Arc<Vec<u8>>),
    Materialized,
}

#[derive(Debug)]
struct ObservedRange {
    offset: u64,
    bytes: Vec<u8>,
}

#[derive(Debug)]
pub(super) struct IndexedFace {
    pub(super) index: u32,
    pub(super) names: Vec<NameOccurrence>,
}

#[derive(Debug)]
pub(super) struct IndexedFontFile {
    root_ordinal: u32,
    pub(super) path: PathBuf,
    source: IndexedSource,
    ranges: Vec<ObservedRange>,
    pub(super) faces: Vec<IndexedFace>,
    pub(super) issues: Vec<(Option<u32>, &'static str)>,
}

/// File and generated-byte fixtures use exactly this range reader and indexing
/// algorithm. A range outside the observed length is stable content rejection;
/// an actual seek/read failure or short read remains an operation error.
struct Ranges<R> {
    reader: R,
    length: u64,
    path: PathBuf,
    observed: Vec<ObservedRange>,
}

impl<R: Read + Seek> Ranges<R> {
    fn read(
        &mut self,
        offset: u64,
        length: usize,
        mut stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    ) -> Result<Option<Vec<u8>>, FontError> {
        let end = offset
            .checked_add(
                u64::try_from(length).map_err(|_| FontError::Arithmetic("font metadata range"))?,
            )
            .ok_or(FontError::Arithmetic("font metadata range"))?;
        if end > self.length {
            return Ok(None);
        }
        let mut bytes = Vec::new();
        reserve_prepared_or_local(
            &mut bytes,
            length,
            "font metadata range",
            stage.as_deref_mut(),
        )?;
        bytes.resize(length, 0);
        self.reader
            .seek(SeekFrom::Start(offset))
            .map_err(|source| FontError::Io {
                operation: "seek font metadata",
                path: self.path.clone(),
                source,
            })?;
        let mut cursor = 0;
        while cursor < bytes.len() {
            let read = self
                .reader
                .read(&mut bytes[cursor..])
                .map_err(|source| FontError::Io {
                    operation: "read font metadata",
                    path: self.path.clone(),
                    source,
                })?;
            if read == 0 {
                return Err(FontError::ChangedFile(self.path.clone()));
            }
            #[cfg(test)]
            LOAD_OBSERVATIONS.with_borrow_mut(|observed| {
                *observed
                    .metadata_bytes
                    .entry(self.path.clone())
                    .or_default() += read;
            });
            cursor += read;
        }
        let mut retained = Vec::new();
        reserve_prepared_or_local(
            &mut retained,
            length,
            "observed font metadata",
            stage.as_deref_mut(),
        )?;
        retained.extend_from_slice(&bytes);
        reserve_prepared_or_local(&mut self.observed, 1, "observed font ranges", stage)?;
        self.observed.push(ObservedRange {
            offset,
            bytes: retained,
        });
        Ok(Some(bytes))
    }
}

fn word(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes([bytes[offset], bytes[offset + 1]])
}

/// Project a name table without reading its unconsumed string payload. Original
/// records/ordinals and storage offsets stay intact; unread storage is only a
/// temporary zero-filled view, never a Face source or repaired font. Only actual
/// Unicode family strings in that view are decoded by the common name decoder.
fn family_names<R: Read + Seek>(
    input: &mut Ranges<R>,
    table: ttf_parser::TableRecord,
    mut stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<Vec<NameOccurrence>, FontError> {
    let start = u64::from(table.offset);
    let length = usize::try_from(table.length)
        .map_err(|_| FontError::Arithmetic("font name table length"))?;
    if start + u64::from(table.length) > input.length || length < 6 {
        return Ok(Vec::new());
    }
    let Some(header) = input.read(start, 6, stage.as_deref_mut())? else {
        return Ok(Vec::new());
    };
    let records_start = match word(&header, 0) {
        0 => 6,
        1 if length >= 8 => {
            let Some(count) = input.read(start + 6, 2, stage.as_deref_mut())? else {
                return Ok(Vec::new());
            };
            let Some(skipped) = word(&count, 0).checked_mul(4) else {
                return Ok(Vec::new());
            };
            8 + usize::from(skipped)
        }
        _ => return Ok(Vec::new()),
    };
    let records_length = usize::from(word(&header, 2)) * 12;
    let records_end = records_start + records_length;
    let storage_start = records_end.max(usize::from(word(&header, 4)));
    if storage_start > length {
        return Ok(Vec::new());
    }
    let Some(records) = input.read(
        start + records_start as u64,
        records_length,
        stage.as_deref_mut(),
    )?
    else {
        return Ok(Vec::new());
    };
    let mut storage_length = 0;
    for record in records.chunks_exact(12) {
        let end = usize::from(word(record, 10)) + usize::from(word(record, 8));
        if matches!(word(record, 6), 1 | 16) && end <= length - storage_start {
            storage_length = storage_length.max(end);
        }
    }
    let mut view = Vec::new();
    reserve_prepared_or_local(
        &mut view,
        storage_start + storage_length,
        "font family name view",
        stage.as_deref_mut(),
    )?;
    view.resize(storage_start + storage_length, 0);
    view[..6].copy_from_slice(&header);
    if records_start != 6 {
        // The locked parser consumes this count but only skips langTagRecords.
        let skipped = u16::try_from((records_start - 8) / 4)
            .map_err(|_| FontError::Arithmetic("font name language tags"))?;
        view[6..8].copy_from_slice(&skipped.to_be_bytes());
    }
    view[records_start..records_end].copy_from_slice(&records);
    for (ordinal, record) in records.chunks_exact(12).enumerate() {
        if !matches!(word(record, 6), 1 | 16) {
            continue;
        }
        let ordinal = u16::try_from(ordinal)
            .map_err(|_| FontError::Arithmetic("font name record ordinal"))?;
        let Some(name) =
            ttf_parser::name::Table::parse(&view).and_then(|table| table.names.get(ordinal))
        else {
            continue;
        };
        if !name.is_unicode() {
            continue;
        }
        let offset = storage_start + usize::from(word(record, 10));
        let count = usize::from(word(record, 8));
        if offset + count > view.len() {
            continue;
        }
        let Some(bytes) = input.read(start + offset as u64, count, stage.as_deref_mut())? else {
            continue;
        };
        view[offset..offset + count].copy_from_slice(&bytes);
    }
    decode_family_names(
        ttf_parser::name::Table::parse(&view)
            .map_or_else(ttf_parser::name::Names::default, |table| table.names),
        stage,
    )
}

fn index<R: Read + Seek>(
    mut input: Ranges<R>,
    root_ordinal: u32,
    source: IndexedSource,
    mut stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<IndexedFontFile, FontError> {
    let mut faces = Vec::new();
    let mut issues = Vec::new();
    let header = input.read(0, 12, stage.as_deref_mut())?.unwrap_or_default();
    let collection = header.get(..4) == Some(b"ttcf");
    let count = if collection {
        ttf_parser::fonts_in_collection(&header)
            .filter(|count| 12 + u64::from(*count) * 4 <= input.length)
    } else {
        Some(1)
    };
    let mut offsets = Vec::new();
    if let Some(count) = count {
        if collection {
            offsets = input
                .read(
                    12,
                    usize::try_from(u64::from(count) * 4)
                        .map_err(|_| FontError::Arithmetic("font collection offsets"))?,
                    stage.as_deref_mut(),
                )?
                .ok_or(FontError::Arithmetic("bounded font collection offsets"))?;
        }
        for face_index in 0..count {
            let offset = if collection {
                let ordinal = usize::try_from(face_index)
                    .map_err(|_| FontError::Arithmetic("font collection face index"))?
                    * 4;
                u64::from(u32::from_be_bytes(
                    offsets[ordinal..ordinal + 4]
                        .try_into()
                        .map_err(|_| FontError::Arithmetic("font collection face offset"))?,
                ))
            } else {
                0
            };
            let names = if !collection || offset >= 12 + u64::from(count) * 4 {
                index_face(&mut input, offset, stage.as_deref_mut())?
            } else {
                None
            };
            if let Some(names) = names.filter(|names| !names.is_empty()) {
                reserve_prepared_or_local(
                    &mut faces,
                    1,
                    "indexed font faces",
                    stage.as_deref_mut(),
                )?;
                faces.push(IndexedFace {
                    index: face_index,
                    names,
                });
            } else {
                reserve_prepared_or_local(
                    &mut issues,
                    1,
                    "font index diagnostics",
                    stage.as_deref_mut(),
                )?;
                issues.push((Some(face_index), "unaddressable face metadata"));
            }
        }
    } else {
        reserve_prepared_or_local(&mut issues, 1, "font index diagnostics", stage)?;
        issues.push((None, "truncated collection offset array"));
    }
    Ok(IndexedFontFile {
        root_ordinal,
        path: input.path,
        source,
        ranges: input.observed,
        faces,
        issues,
    })
}

fn index_face<R: Read + Seek>(
    input: &mut Ranges<R>,
    offset: u64,
    mut stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<Option<Vec<NameOccurrence>>, FontError> {
    let Some(header) = input.read(offset, 12, stage.as_deref_mut())? else {
        return Ok(None);
    };
    if header.get(..4) == Some(b"ttcf") {
        return Ok(None);
    }
    let length = usize::from(word(&header, 4)) * 16;
    let Some(records) = input.read(offset + 12, length, stage.as_deref_mut())? else {
        return Ok(None);
    };
    let mut directory = Vec::new();
    reserve_prepared_or_local(
        &mut directory,
        12 + length,
        "font directory view",
        stage.as_deref_mut(),
    )?;
    directory.extend_from_slice(&header);
    directory.extend_from_slice(&records);
    let Ok(raw) = ttf_parser::RawFace::parse(&directory, 0) else {
        return Ok(None);
    };
    // Face::collect_tables uses the last declared record, even when its range is
    // invalid. Do not substitute RawFace::table's binary-search name lookup.
    let name = raw
        .table_records
        .into_iter()
        .filter(|record| record.tag == ttf_parser::Tag::from_bytes(b"name"))
        .last();
    match name {
        Some(table) => family_names(input, table, stage).map(Some),
        None => Ok(None),
    }
}

fn open(path: &Path) -> Result<DiscoveryFontHandle, FontError> {
    let file = File::open(path).map_err(|source| FontError::Io {
        operation: "open font source",
        path: path.to_path_buf(),
        source,
    })?;
    #[cfg(test)]
    DISCOVERY_HANDLES.with(|state| {
        let mut counts = state.get();
        counts[0] += 1;
        counts[1] = counts[1].max(counts[0]);
        counts[2] += 1;
        state.set(counts);
    });
    Ok(DiscoveryFontHandle { file })
}

pub(super) fn index_file(
    root_ordinal: u32,
    path: PathBuf,
    stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<IndexedFontFile, FontError> {
    let mut handle = open(&path)?;
    let before = stamp(&handle.file, &path)?;
    #[cfg(test)]
    DISCOVERY_AFTER_ADMISSION.with(|hook| {
        if let Some(hook) = hook.take() {
            hook(&path);
        }
    });
    let indexed = index(
        Ranges {
            reader: &mut handle.file,
            length: before.length,
            path: path.clone(),
            observed: Vec::new(),
        },
        root_ordinal,
        IndexedSource::File(before),
        stage,
    )?;
    if let IndexedSource::File(before) = &indexed.source {
        if *before != stamp(&handle.file, &path)? {
            return Err(FontError::ChangedFile(path));
        }
    }
    Ok(indexed)
}

#[cfg(test)]
pub(super) fn generated(bytes: Vec<u8>) -> Result<IndexedFontFile, FontError> {
    let bytes = Arc::new(bytes);
    let path = PathBuf::from("generated-font.ttf");
    index(
        Ranges {
            reader: std::io::Cursor::new(bytes.as_slice()),
            length: bytes.len() as u64,
            path,
            observed: Vec::new(),
        },
        0,
        IndexedSource::Generated(Arc::clone(&bytes)),
        None,
    )
}

impl IndexedFontFile {
    pub(super) fn is_materialized(&self) -> bool {
        matches!(self.source, IndexedSource::Materialized)
    }

    pub(super) fn finish_materialization(&mut self) {
        self.source = IndexedSource::Materialized;
        self.ranges.clear();
    }

    pub(super) fn matches(&self, names: &[(String, bool)]) -> bool {
        self.faces.iter().any(|face| {
            face.names
                .iter()
                .any(|name| names.iter().any(|(resolved, _)| name.folded == *resolved))
        })
    }

    pub(super) fn materialize(
        &self,
        stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    ) -> Result<RetainedFontFile, FontError> {
        let (bytes, sha256) = match &self.source {
            IndexedSource::File(before) => {
                let mut handle = open(&self.path)?;
                if *before != stamp(&handle.file, &self.path)? {
                    return Err(FontError::ChangedFile(self.path.clone()));
                }
                let length = usize::try_from(before.length)
                    .map_err(|_| FontError::Arithmetic("font file length"))?;
                let mut bytes = Vec::new();
                reserve_prepared_or_local(&mut bytes, length, "candidate font bytes", stage)?;
                let sha256 =
                    read_exact_font_bytes(&mut handle.file, &self.path, length, &mut bytes)?;
                if *before != stamp(&handle.file, &self.path)? {
                    return Err(FontError::ChangedFile(self.path.clone()));
                }
                (Arc::new(bytes), sha256)
            }
            #[cfg(test)]
            IndexedSource::Generated(bytes) => {
                let mut retained = Vec::new();
                reserve_prepared_or_local(
                    &mut retained,
                    bytes.len(),
                    "candidate font bytes",
                    stage,
                )?;
                let sha256 = read_exact_font_bytes(
                    &mut bytes.as_slice(),
                    &self.path,
                    bytes.len(),
                    &mut retained,
                )?;
                (Arc::new(retained), sha256)
            }
            IndexedSource::Materialized => {
                return Err(FontError::Arithmetic("font file materialized twice"));
            }
        };
        for range in &self.ranges {
            let start = usize::try_from(range.offset)
                .map_err(|_| FontError::Arithmetic("observed font range offset"))?;
            let end = start
                .checked_add(range.bytes.len())
                .ok_or(FontError::Arithmetic("observed font range end"))?;
            if bytes.get(start..end) != Some(range.bytes.as_slice()) {
                return Err(FontError::ChangedFile(self.path.clone()));
            }
        }
        #[cfg(test)]
        LOAD_OBSERVATIONS.with_borrow_mut(|observed| {
            observed.materialized.push((self.path.clone(), bytes.len()));
        });
        Ok(RetainedFontFile {
            root_ordinal: self.root_ordinal,
            canonical_path: self.path.clone(),
            bytes,
            sha256,
        })
    }
}
