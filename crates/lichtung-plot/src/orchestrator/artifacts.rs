//! Atomic render-artifact staging, publication, and cleanup.

use std::io::Write;
use std::path::{Path, PathBuf};

#[cfg(test)]
use std::sync::atomic::{AtomicU64, Ordering};

use crate::Error;

const STAGING_PREFIX: &str = ".lichtung-tmp-";
const STAGING_RANDOM_BYTES: usize = 16;

#[cfg(test)]
pub(super) static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[cfg(unix)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StagingPathState {
    Owned,
    Missing,
    Foreign,
}

#[derive(Debug)]
struct OwnedArtifactStaging {
    destination: PathBuf,
    staged: Option<tempfile::NamedTempFile>,
}

impl OwnedArtifactStaging {
    fn create(path: &Path) -> Result<Self, Error> {
        let destination = publication_destination(path)?;
        let parent = destination
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let staged = create_staging_file(parent)?;
        Ok(Self {
            destination,
            staged: Some(staged),
        })
    }

    fn file_mut(&mut self) -> Result<&mut std::fs::File, Error> {
        self.staged
            .as_mut()
            .map(tempfile::NamedTempFile::as_file_mut)
            .ok_or_else(|| Error::Render("artifact staging is no longer active".into()))
    }

    #[cfg(any(unix, test))]
    fn path(&self) -> Result<&Path, Error> {
        self.staged
            .as_ref()
            .map(tempfile::NamedTempFile::path)
            .ok_or_else(|| Error::Render("artifact staging is no longer active".into()))
    }

    #[cfg(unix)]
    fn path_state(&self) -> Result<StagingPathState, Error> {
        let staged = self
            .staged
            .as_ref()
            .ok_or_else(|| Error::Render("artifact staging is no longer active".into()))?;
        let path_metadata = match std::fs::symlink_metadata(staged.path()) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(StagingPathState::Missing);
            }
            Err(error) => return Err(error.into()),
        };
        let file_metadata = staged.as_file().metadata()?;
        if metadata_identifies_same_file(&file_metadata, &path_metadata) {
            Ok(StagingPathState::Owned)
        } else {
            Ok(StagingPathState::Foreign)
        }
    }

    #[cfg(unix)]
    fn commit(&mut self) -> Result<(), Error> {
        if self.path_state()? != StagingPathState::Owned {
            return Err(Error::Render(
                "artifact staging path no longer identifies the owned staging file".into(),
            ));
        }
        let staged = self
            .staged
            .take()
            .ok_or_else(|| Error::Render("artifact staging is no longer active".into()))?;
        match staged.persist(&self.destination) {
            Ok(_) => Ok(()),
            Err(tempfile::PersistError { error, file }) => {
                self.staged = Some(file);
                Err(error.into())
            }
        }
    }

    #[cfg(windows)]
    fn commit(&mut self) -> Result<(), Error> {
        let staged = self
            .staged
            .as_ref()
            .ok_or_else(|| Error::Render("artifact staging is no longer active".into()))?;
        windows_owned::replace(staged.as_file(), &self.destination)?;
        self.disarm();
        Ok(())
    }

    #[cfg(not(any(unix, windows)))]
    fn commit(&mut self) -> Result<(), Error> {
        Err(Error::Render(
            "atomic artifact publication is unsupported on this host".into(),
        ))
    }

    #[cfg(unix)]
    fn cleanup(&mut self) -> Result<(), Error> {
        match self.path_state()? {
            StagingPathState::Owned => {
                let path = self.path()?.to_path_buf();
                remove_if_present(&path)?;
                self.disarm();
            }
            StagingPathState::Missing | StagingPathState::Foreign => self.disarm(),
        }
        Ok(())
    }

    #[cfg(windows)]
    fn cleanup(&mut self) -> Result<(), Error> {
        let staged = self
            .staged
            .as_ref()
            .ok_or_else(|| Error::Render("artifact staging is no longer active".into()))?;
        windows_owned::remove(staged.as_file())?;
        self.disarm();
        Ok(())
    }

    #[cfg(not(any(unix, windows)))]
    fn cleanup(&mut self) -> Result<(), Error> {
        self.disarm();
        Ok(())
    }

    fn disarm(&mut self) {
        if let Some(mut staged) = self.staged.take() {
            staged.disable_cleanup(true);
        }
    }
}

#[cfg(unix)]
fn create_staging_file(parent: &Path) -> std::io::Result<tempfile::NamedTempFile> {
    use std::os::unix::fs::PermissionsExt;

    tempfile::Builder::new()
        .prefix(STAGING_PREFIX)
        .rand_bytes(STAGING_RANDOM_BYTES)
        .permissions(std::fs::Permissions::from_mode(0o666))
        .tempfile_in(parent)
}

#[cfg(windows)]
fn create_staging_file(parent: &Path) -> std::io::Result<tempfile::NamedTempFile> {
    use std::os::windows::fs::OpenOptionsExt;
    use windows_sys::Win32::Storage::FileSystem::{DELETE, FILE_GENERIC_READ, FILE_GENERIC_WRITE};

    tempfile::Builder::new()
        .prefix(STAGING_PREFIX)
        .rand_bytes(STAGING_RANDOM_BYTES)
        .make_in(parent, |path| {
            std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .access_mode(FILE_GENERIC_READ | FILE_GENERIC_WRITE | DELETE)
                .share_mode(0)
                .create_new(true)
                .open(path)
        })
}

#[cfg(not(any(unix, windows)))]
fn create_staging_file(parent: &Path) -> std::io::Result<tempfile::NamedTempFile> {
    tempfile::Builder::new()
        .prefix(STAGING_PREFIX)
        .rand_bytes(STAGING_RANDOM_BYTES)
        .tempfile_in(parent)
}

impl Drop for OwnedArtifactStaging {
    fn drop(&mut self) {
        if self.staged.is_some() {
            let _ = self.cleanup();
            self.disarm();
        }
    }
}

#[cfg(unix)]
fn metadata_identifies_same_file(file: &std::fs::Metadata, path: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;

    !path.file_type().is_symlink() && file.dev() == path.dev() && file.ino() == path.ino()
}

#[cfg(windows)]
#[allow(unsafe_code)]
mod windows_owned {
    use std::mem::{self, MaybeUninit};
    use std::os::windows::ffi::OsStrExt;
    use std::os::windows::io::AsRawHandle;
    use std::path::Path;

    use windows_sys::Win32::Storage::FileSystem::{
        FileDispositionInfo, FileRenameInfo, SetFileInformationByHandle, FILE_DISPOSITION_INFO,
        FILE_RENAME_INFO,
    };

    pub(super) fn replace(file: &std::fs::File, destination: &Path) -> std::io::Result<()> {
        let (mut buffer, buffer_bytes) = rename_information(destination)?;
        // SAFETY: `rename_information` supplies an aligned buffer with the
        // populated header and NUL-terminated filename, live for this
        // synchronous Win32 call.
        if unsafe {
            SetFileInformationByHandle(
                file.as_raw_handle(),
                FileRenameInfo,
                buffer.as_mut_ptr().cast(),
                buffer_bytes,
            )
        } == 0
        {
            return Err(std::io::Error::last_os_error());
        }
        Ok(())
    }

    fn rename_information(
        destination: &Path,
    ) -> std::io::Result<(Vec<MaybeUninit<FILE_RENAME_INFO>>, u32)> {
        let name = destination.as_os_str().encode_wide().collect::<Vec<_>>();
        let name_bytes = name
            .len()
            .checked_mul(mem::size_of::<u16>())
            .and_then(|length| u32::try_from(length).ok())
            .ok_or_else(path_too_long)?;
        let buffer_bytes = mem::offset_of!(FILE_RENAME_INFO, FileName)
            .checked_add(name_bytes as usize)
            .and_then(|length| length.checked_add(mem::size_of::<u16>()))
            .ok_or_else(path_too_long)?;
        let buffer_bytes = u32::try_from(buffer_bytes).map_err(|_| path_too_long())?;
        let entries = (buffer_bytes as usize).div_ceil(mem::size_of::<FILE_RENAME_INFO>());
        let mut buffer = vec![MaybeUninit::<FILE_RENAME_INFO>::uninit(); entries];
        let info = buffer.as_mut_ptr().cast::<FILE_RENAME_INFO>();

        // SAFETY: `buffer` is aligned for FILE_RENAME_INFO and large enough for
        // its header, UTF-16 name and terminating NUL. Initialize the entire
        // allocation, including padding, before writing the counted name.
        unsafe {
            std::ptr::write_bytes(info, 0, buffer.len());
            std::ptr::addr_of_mut!((*info).Anonymous.ReplaceIfExists).write(true);
            std::ptr::addr_of_mut!((*info).RootDirectory).write(std::ptr::null_mut());
            std::ptr::addr_of_mut!((*info).FileNameLength).write(name_bytes);
            let name_destination = std::ptr::addr_of_mut!((*info).FileName).cast::<u16>();
            std::ptr::copy_nonoverlapping(name.as_ptr(), name_destination, name.len());
            // Win32 path conversion reads to NUL even with FileNameLength set.
            name_destination.add(name.len()).write(0);
        }
        Ok((buffer, buffer_bytes))
    }

    pub(super) fn remove(file: &std::fs::File) -> std::io::Result<()> {
        let disposition = FILE_DISPOSITION_INFO { DeleteFile: true };
        let disposition_bytes = u32::try_from(mem::size_of::<FILE_DISPOSITION_INFO>())
            .map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidInput))?;
        // SAFETY: `disposition` has the structure and size required by
        // FileDispositionInfo, and `file` keeps the source handle live.
        if unsafe {
            SetFileInformationByHandle(
                file.as_raw_handle(),
                FileDispositionInfo,
                std::ptr::from_ref(&disposition).cast(),
                disposition_bytes,
            )
        } == 0
        {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    fn path_too_long() -> std::io::Error {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "render output path is too long for Windows atomic replacement",
        )
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn rename_information_terminates_utf16_outside_the_counted_name() {
            let boundary_units = (mem::size_of::<FILE_RENAME_INFO>()
                - mem::offset_of!(FILE_RENAME_INFO, FileName))
                / mem::size_of::<u16>();
            let boundary_name = "a".repeat(boundary_units);
            for name in [boundary_name.as_str(), "图🦀.svg"] {
                let expected = std::ffi::OsStr::new(name).encode_wide().collect::<Vec<_>>();
                let (mut buffer, buffer_bytes) = rename_information(Path::new(name)).unwrap();
                let expected_bytes = mem::offset_of!(FILE_RENAME_INFO, FileName)
                    + (expected.len() + 1) * mem::size_of::<u16>();
                assert_eq!(buffer_bytes as usize, expected_bytes);
                assert!(buffer.len() * mem::size_of::<FILE_RENAME_INFO>() >= expected_bytes);

                // SAFETY: the builder initializes the header and complete
                // UTF-16 name, including the separately sized terminator.
                let info = unsafe { &*buffer.as_mut_ptr().cast::<FILE_RENAME_INFO>() };
                assert_eq!(unsafe { info.Anonymous.Flags }, 1);
                assert!(info.RootDirectory.is_null());
                assert_eq!(
                    info.FileNameLength as usize,
                    expected.len() * mem::size_of::<u16>()
                );
                let actual = unsafe {
                    std::slice::from_raw_parts(
                        std::ptr::addr_of!(info.FileName).cast::<u16>(),
                        expected.len() + 1,
                    )
                };
                assert_eq!(&actual[..expected.len()], expected);
                assert_eq!(actual.last(), Some(&0));
            }
        }
    }
}

fn publication_destination(path: &Path) -> Result<PathBuf, Error> {
    let file_name = path.file_name().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("render output path {} has no file name", path.display()),
        )
    })?;
    #[cfg(windows)]
    if windows_invalid_or_reserved_file_name(file_name) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "render output path uses an invalid or reserved Windows file name",
        )
        .into());
    }
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    Ok(std::fs::canonicalize(parent)?.join(file_name))
}

#[cfg(windows)]
fn windows_invalid_or_reserved_file_name(file_name: &std::ffi::OsStr) -> bool {
    use std::os::windows::ffi::OsStrExt;

    let units = file_name.encode_wide().collect::<Vec<_>>();
    units.is_empty()
        || units
            .iter()
            .any(|unit| matches!(*unit, 0..=31 | 34 | 42 | 47 | 58 | 60 | 62 | 63 | 92 | 124))
        || units.last().is_some_and(|unit| matches!(*unit, 32 | 46))
        || windows_reserved_file_name_units(&units)
}

#[cfg(windows)]
fn windows_reserved_file_name_units(file_name: &[u16]) -> bool {
    let stem_end = file_name
        .iter()
        .position(|unit| matches!(*unit, 46 | 58))
        .unwrap_or(file_name.len());
    let mut stem = &file_name[..stem_end];
    while stem.last().is_some_and(|unit| matches!(*unit, 32 | 46)) {
        stem = &stem[..stem.len() - 1];
    }
    ["CON", "PRN", "AUX", "NUL", "CONIN$", "CONOUT$", "CLOCK$"]
        .iter()
        .any(|name| windows_stem_eq(stem, name))
        || (stem.len() == 4
            && ["COM", "LPT"]
                .iter()
                .any(|prefix| windows_stem_eq(&stem[..3], prefix))
            && matches!(stem[3], 49..=57 | 0x00B9 | 0x00B2 | 0x00B3))
}

#[cfg(windows)]
fn windows_stem_eq(stem: &[u16], expected: &str) -> bool {
    stem.len() == expected.len()
        && stem.iter().zip(expected.bytes()).all(|(actual, expected)| {
            let actual = if matches!(*actual, 97..=122) {
                *actual - 32
            } else {
                *actual
            };
            actual == u16::from(expected)
        })
}

pub(crate) fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    stage_and_publish(path, |staged| staged.write_all(bytes).map_err(Error::Io))
}

fn stage_and_publish(
    path: &Path,
    write: impl FnOnce(&mut std::fs::File) -> Result<(), Error>,
) -> Result<(), Error> {
    let mut staging = OwnedArtifactStaging::create(path)?;
    let result = match staging.file_mut() {
        Ok(file) => write(file),
        Err(error) => Err(error),
    }
    .and_then(|()| staging.commit());
    match result {
        Ok(()) => Ok(()),
        Err(primary) => {
            let cleanup = staging.cleanup();
            crate::render::backend::finish_scope(
                Err(primary),
                cleanup,
                "temporary render artifacts",
            )
        }
    }
}

#[cfg(unix)]
fn remove_if_present(path: &Path) -> std::io::Result<()> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_directory(name: &str) -> PathBuf {
        let sequence = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let directory = std::env::temp_dir().join(format!(
            "lichtung-plot-artifacts-{name}-{}-{sequence}",
            std::process::id()
        ));
        std::fs::create_dir(&directory).unwrap();
        directory
    }

    #[test]
    fn staged_write_failure_preserves_destination_and_removes_partial_file() {
        let directory = test_directory("write-failure");
        let output = directory.join("plot.svg");
        std::fs::write(&output, "existing artifact").unwrap();

        let result = stage_and_publish(&output, |staged| {
            staged.write_all(b"partial artifact")?;
            Err(std::io::Error::new(std::io::ErrorKind::WriteZero, "staged write failed").into())
        });

        assert!(result
            .unwrap_err()
            .to_string()
            .contains("staged write failed"));
        assert_eq!(
            std::fs::read_to_string(output).unwrap(),
            "existing artifact"
        );
        assert!(std::fs::read_dir(&directory).unwrap().all(|entry| !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(STAGING_PREFIX)));
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn windows_owned_handle_prevents_reopen_and_path_substitution() {
        use std::os::windows::fs::OpenOptionsExt;

        let directory = test_directory("windows-owned-staging");
        let output = directory.join("图🦀.svg");
        let moved = directory.join("moved-staging");
        let mut staging = OwnedArtifactStaging::create(&output).unwrap();
        let staged_path = staging.path().unwrap().to_path_buf();

        assert!(std::fs::OpenOptions::new()
            .write(true)
            .open(&staged_path)
            .is_err());
        assert!(std::fs::rename(&staged_path, &moved).is_err());
        staging
            .file_mut()
            .unwrap()
            .write_all(b"owned staging bytes")
            .unwrap();
        staging.commit().unwrap();

        assert_eq!(
            std::fs::read_to_string(&output).unwrap(),
            "owned staging bytes"
        );
        assert!(!staged_path.exists() && !moved.exists());

        write_atomic(&output, b"replacement artifact").unwrap();
        assert_eq!(
            std::fs::read_to_string(&output).unwrap(),
            "replacement artifact"
        );

        let locked_destination = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(&output)
            .unwrap();
        assert!(write_atomic(&output, b"must not be published").is_err());
        drop(locked_destination);
        assert_eq!(
            std::fs::read_to_string(&output).unwrap(),
            "replacement artifact"
        );
        assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 1);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn substituted_staging_path_cannot_redirect_write_or_commit() {
        use std::os::unix::fs::symlink;

        let directory = test_directory("staging-substitution");
        let output = directory.join("plot.svg");
        let foreign = directory.join("foreign");
        std::fs::write(&output, "existing artifact").unwrap();
        std::fs::write(&foreign, "foreign artifact").unwrap();
        let mut staging = OwnedArtifactStaging::create(&output).unwrap();
        let staged_path = staging.path().unwrap().to_path_buf();
        std::fs::remove_file(&staged_path).unwrap();
        symlink(&foreign, &staged_path).unwrap();

        staging
            .file_mut()
            .unwrap()
            .write_all(b"owned staging bytes")
            .unwrap();
        let error = staging.commit().unwrap_err();
        staging.cleanup().unwrap();
        drop(staging);

        assert!(error
            .to_string()
            .contains("no longer identifies the owned staging file"));
        assert_eq!(
            std::fs::read_to_string(output).unwrap(),
            "existing artifact"
        );
        assert_eq!(
            std::fs::read_to_string(foreign).unwrap(),
            "foreign artifact"
        );
        assert!(std::fs::symlink_metadata(staged_path)
            .unwrap()
            .file_type()
            .is_symlink());
        std::fs::remove_dir_all(directory).unwrap();
    }
}
