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

#[cfg(test)]
thread_local! {
    static REMOVE_FAILURES: std::cell::RefCell<Vec<(PathBuf, usize)>> = const { std::cell::RefCell::new(Vec::new()) };
    static REMOVE_ATTEMPTS: std::cell::RefCell<Vec<PathBuf>> = const { std::cell::RefCell::new(Vec::new()) };
}

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
        #[cfg(test)]
        if injected_remove_failure(staged.path()) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "injected temporary artifact removal failure",
            )
            .into());
        }
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
        let name = destination.as_os_str().encode_wide().collect::<Vec<_>>();
        let name_bytes = name
            .len()
            .checked_mul(mem::size_of::<u16>())
            .and_then(|length| u32::try_from(length).ok())
            .ok_or_else(path_too_long)?;
        let buffer_bytes = mem::offset_of!(FILE_RENAME_INFO, FileName)
            .checked_add(name_bytes as usize)
            .ok_or_else(path_too_long)?;
        let entries = buffer_bytes.div_ceil(mem::size_of::<FILE_RENAME_INFO>());
        let mut buffer = vec![MaybeUninit::<FILE_RENAME_INFO>::uninit(); entries];
        let info = buffer.as_mut_ptr().cast::<FILE_RENAME_INFO>();
        let buffer_bytes = u32::try_from(buffer_bytes).map_err(|_| path_too_long())?;

        // SAFETY: `buffer` is aligned for FILE_RENAME_INFO and large enough for
        // its header plus the exact trailing UTF-16 name. The handle and buffer
        // remain live for the synchronous Win32 call.
        unsafe {
            std::ptr::addr_of_mut!((*info).Anonymous.ReplaceIfExists).write(true);
            std::ptr::addr_of_mut!((*info).RootDirectory).write(std::ptr::null_mut());
            std::ptr::addr_of_mut!((*info).FileNameLength).write(name_bytes);
            let name_destination = std::ptr::addr_of_mut!((*info).FileName).cast::<u16>();
            std::ptr::copy_nonoverlapping(name.as_ptr(), name_destination, name.len());
            if SetFileInformationByHandle(
                file.as_raw_handle(),
                FileRenameInfo,
                info.cast(),
                buffer_bytes,
            ) == 0
            {
                return Err(std::io::Error::last_os_error());
            }
        }
        Ok(())
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
    #[cfg(test)]
    let _timer = super::measurements::Timer::new(4);
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

#[cfg(test)]
fn injected_remove_failure(path: &Path) -> bool {
    REMOVE_ATTEMPTS.with(|attempts| attempts.borrow_mut().push(path.to_path_buf()));
    REMOVE_FAILURES.with(|failures| {
        let mut failures = failures.borrow_mut();
        let Some((_, remaining)) = failures
            .iter_mut()
            .find(|(candidate, remaining)| candidate == path && *remaining > 0)
        else {
            return false;
        };
        *remaining -= 1;
        true
    })
}

#[cfg(unix)]
fn remove_if_present(path: &Path) -> std::io::Result<()> {
    #[cfg(test)]
    if injected_remove_failure(path) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "injected temporary artifact removal failure",
        ));
    }
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

    fn reset_removal_evidence() {
        REMOVE_FAILURES.with(|failures| failures.borrow_mut().clear());
        REMOVE_ATTEMPTS.with(|attempts| attempts.borrow_mut().clear());
    }

    fn fail_next_removal(path: &Path) {
        REMOVE_FAILURES.with(|failures| failures.borrow_mut().push((path.to_path_buf(), 1)));
    }

    fn removal_attempts(path: &Path) -> usize {
        REMOVE_ATTEMPTS.with(|attempts| {
            attempts
                .borrow()
                .iter()
                .filter(|candidate| *candidate == path)
                .count()
        })
    }

    #[test]
    fn staging_leaf_is_fixed_sibling_and_destination_independent() {
        let directory = test_directory("owned-staging-leaf");
        let output = directory.join("a-very-long-destination-name.svg");
        let staging = OwnedArtifactStaging::create(&output).unwrap();
        let path = staging.path().unwrap().to_path_buf();
        let leaf = path.file_name().unwrap().to_string_lossy();

        assert_eq!(path.parent(), Some(directory.as_path()));
        assert!(leaf.starts_with(STAGING_PREFIX));
        assert_eq!(leaf.len(), STAGING_PREFIX.len() + STAGING_RANDOM_BYTES);
        assert!(!leaf.contains("destination") && !leaf.ends_with(".svg"));
        drop(staging);
        assert!(!path.exists());
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn reservation_rejects_missing_leaf_and_parent() {
        let error = OwnedArtifactStaging::create(Path::new("/")).unwrap_err();
        assert!(error.to_string().contains("has no file name"));
        let directory = test_directory("reservation-io-failure");
        let output = directory.join("missing").join("plot.svg");
        let error = OwnedArtifactStaging::create(&output).unwrap_err();
        assert!(matches!(error, Error::Io(_)));
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn relative_destination_is_anchored_before_staging() {
        let destination = publication_destination(Path::new("plot.svg")).unwrap();

        assert!(destination.is_absolute());
        assert_eq!(
            destination,
            std::fs::canonicalize(".").unwrap().join("plot.svg")
        );
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

    #[test]
    fn cleanup_failure_retains_primary_diagnostic_and_drop_retries_once() {
        reset_removal_evidence();
        let directory = test_directory("cleanup-failure");
        let output = directory.join("plot.svg");
        let mut staging = OwnedArtifactStaging::create(&output).unwrap();
        let staged_path = staging.path().unwrap().to_path_buf();
        staging.file_mut().unwrap().write_all(b"partial").unwrap();
        fail_next_removal(&staged_path);

        let cleanup = staging.cleanup();
        let result: Result<(), Error> = crate::render::backend::finish_scope(
            Err(Error::Render("primary publication failure".into())),
            cleanup,
            "temporary render artifacts",
        );
        let error = result.unwrap_err().to_string();
        drop(staging);

        assert!(error.contains("primary publication failure"));
        assert!(error.contains("injected temporary artifact removal failure"));
        assert_eq!(removal_attempts(&staged_path), 2);
        assert!(!staged_path.exists());
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn successful_commit_disarms_cleanup_before_returning() {
        reset_removal_evidence();
        let directory = test_directory("commit-boundary");
        let output = directory.join("plot.svg");
        let mut staging = OwnedArtifactStaging::create(&output).unwrap();
        let staged_path = staging.path().unwrap().to_path_buf();
        staging
            .file_mut()
            .unwrap()
            .write_all(b"replacement artifact")
            .unwrap();

        staging.commit().unwrap();
        std::fs::write(&staged_path, "new occupant").unwrap();
        drop(staging);

        assert_eq!(
            std::fs::read_to_string(output).unwrap(),
            "replacement artifact"
        );
        assert_eq!(
            std::fs::read_to_string(&staged_path).unwrap(),
            "new occupant"
        );
        assert_eq!(removal_attempts(&staged_path), 0);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn publication_failure_preserves_destination_and_removes_staging() {
        let directory = test_directory("publication-failure");
        let output = directory.join("plot.svg");
        std::fs::create_dir(&output).unwrap();
        std::fs::write(output.join("existing"), "existing artifact").unwrap();

        let result = write_atomic(&output, b"replacement artifact");

        assert!(result.is_err());
        assert_eq!(
            std::fs::read_to_string(output.join("existing")).unwrap(),
            "existing artifact"
        );
        assert!(std::fs::read_dir(&directory).unwrap().all(|entry| !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(STAGING_PREFIX)));
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn drop_removes_owned_staging_file_during_unwind() {
        let directory = test_directory("drop-cleanup");
        let output = directory.join("plot.svg");
        let staging = OwnedArtifactStaging::create(&output).unwrap();
        let observed = staging.path().unwrap().to_path_buf();

        let unwind = std::panic::catch_unwind(|| {
            let _staging = staging;
            panic!("simulate transaction unwind");
        });

        assert!(unwind.is_err());
        assert!(!observed.exists());
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn drop_suppresses_final_cleanup_failure() {
        reset_removal_evidence();
        let directory = test_directory("drop-cleanup-failure");
        let output = directory.join("plot.svg");
        let staging = OwnedArtifactStaging::create(&output).unwrap();
        let observed = staging.path().unwrap().to_path_buf();
        fail_next_removal(&observed);

        drop(staging);

        assert_eq!(removal_attempts(&observed), 1);
        assert!(observed.exists());
        reset_removal_evidence();
        std::fs::remove_file(observed).unwrap();
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn component_limit_destination_uses_short_independent_staging_leaf() {
        let directory = test_directory("component-limit");
        let output = directory.join("d".repeat(255));

        write_atomic(&output, b"complete artifact").unwrap();

        assert_eq!(std::fs::read(&output).unwrap(), b"complete artifact");
        assert!(std::fs::read_dir(&directory).unwrap().all(|entry| !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(STAGING_PREFIX)));
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn published_permissions_match_ordinary_file_creation() {
        use std::os::unix::fs::PermissionsExt;

        let directory = test_directory("publication-permissions");
        let reference = directory.join("reference");
        let output = directory.join("plot.svg");
        drop(
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&reference)
                .unwrap(),
        );

        write_atomic(&output, b"complete artifact").unwrap();

        let reference_mode = std::fs::metadata(reference).unwrap().permissions().mode() & 0o777;
        let output_mode = std::fs::metadata(output).unwrap().permissions().mode() & 0o777;
        assert_eq!(output_mode, reference_mode);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn windows_owned_handle_prevents_reopen_and_path_substitution() {
        let directory = test_directory("windows-owned-staging");
        let output = directory.join("plot.svg");
        let moved = directory.join("moved-staging");
        std::fs::write(&output, "existing artifact").unwrap();
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
            std::fs::read_to_string(output).unwrap(),
            "owned staging bytes"
        );
        assert!(!staged_path.exists() && !moved.exists());
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
