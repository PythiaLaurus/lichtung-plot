//! Atomic render-artifact staging, publication, and cleanup.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::Error;

pub(super) static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[cfg(test)]
thread_local! {
    pub(crate) static TEMP_RESERVATION_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static REMOVE_FAILURES: std::cell::RefCell<Vec<(PathBuf, usize)>> = const { std::cell::RefCell::new(Vec::new()) };
    static REMOVE_ATTEMPTS: std::cell::RefCell<Vec<PathBuf>> = const { std::cell::RefCell::new(Vec::new()) };
}

pub(crate) fn reserve_unique_sibling_temp_path(
    path: &Path,
    extension: &str,
) -> Result<PathBuf, Error> {
    #[cfg(test)]
    TEMP_RESERVATION_CALLS.with(|calls| calls.set(calls.get() + 1));
    #[cfg(windows)]
    if !windows_is_verbatim(path)
        && path
            .file_name()
            .is_some_and(windows_invalid_or_reserved_file_name)
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "render output path uses an invalid or reserved Windows file name",
        )
        .into());
    }
    reserve_unique_path(|| unique_sibling_temp_path(path, extension))
}

fn reserve_unique_path(
    mut next_candidate: impl FnMut() -> Result<PathBuf, Error>,
) -> Result<PathBuf, Error> {
    loop {
        let candidate = next_candidate()?;
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(_) => return Ok(candidate),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error.into()),
        }
    }
}

fn unique_sibling_temp_path(path: &Path, extension: &str) -> Result<PathBuf, Error> {
    let file_name = path.file_name().ok_or_else(|| {
        Error::Data(format!(
            "render output path {} has no file name",
            path.display()
        ))
    })?;
    let sequence = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut temp_name = file_name.to_os_string();
    temp_name.push(format!(
        ".lichtung-tmp-{}-{sequence}.{extension}",
        std::process::id()
    ));
    Ok(path.with_file_name(temp_name))
}

pub(crate) fn publish_staged(staged: &Path, final_path: &Path) -> Result<(), Error> {
    #[cfg(not(windows))]
    {
        std::fs::rename(staged, final_path)?;
        Ok(())
    }
    #[cfg(windows)]
    {
        publish_staged_windows(staged, final_path)
    }
}

#[cfg(windows)]
fn publish_staged_windows(staged: &Path, final_path: &Path) -> Result<(), Error> {
    // Canonicalizing the staged file keeps the generated suffix usable near
    // MAX_PATH. Destination extension is limited to leaf names whose ordinary
    // and verbatim Win32 meanings are identical.
    let staged = tempfile::TempPath::try_from_path(std::fs::canonicalize(staged)?)?;
    match staged.persist(windows_atomic_destination_path(final_path)?) {
        Ok(()) => Ok(()),
        Err(tempfile::PathPersistError { error, mut path }) => {
            // The outer transaction owns observable cleanup and its diagnostics.
            path.disable_cleanup(true);
            Err(error.into())
        }
    }
}

#[cfg(windows)]
fn windows_atomic_destination_path(path: &Path) -> std::io::Result<PathBuf> {
    let Some(file_name) = path.file_name() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "artifact path has no file name",
        ));
    };
    let Some(file_name) = file_name.to_str() else {
        return Ok(path.to_path_buf());
    };
    if !windows_verbatim_safe_file_name(file_name) {
        return Ok(path.to_path_buf());
    }
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    Ok(std::fs::canonicalize(parent)?.join(file_name))
}

#[cfg(windows)]
fn windows_verbatim_safe_file_name(file_name: &str) -> bool {
    if file_name.is_empty()
        || !file_name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
        || file_name.ends_with('.')
    {
        return false;
    }
    !windows_reserved_file_name(std::ffi::OsStr::new(file_name))
}

#[cfg(windows)]
fn windows_invalid_or_reserved_file_name(file_name: &std::ffi::OsStr) -> bool {
    use std::os::windows::ffi::OsStrExt;

    let units = file_name.encode_wide().collect::<Vec<_>>();
    units.contains(&u16::from(b':')) || windows_reserved_file_name_units(&units)
}

#[cfg(windows)]
fn windows_reserved_file_name(file_name: &std::ffi::OsStr) -> bool {
    use std::os::windows::ffi::OsStrExt;

    windows_reserved_file_name_units(&file_name.encode_wide().collect::<Vec<_>>())
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

#[cfg(windows)]
fn windows_is_verbatim(path: &Path) -> bool {
    use std::path::{Component, Prefix};

    matches!(
        path.components().next(),
        Some(Component::Prefix(prefix))
            if matches!(
                prefix.kind(),
                Prefix::Verbatim(_) | Prefix::VerbatimUNC(_, _) | Prefix::VerbatimDisk(_)
            )
    )
}

pub(crate) fn write_atomic(path: &Path, extension: &str, bytes: &[u8]) -> Result<(), Error> {
    stage_and_publish(path, extension, |staged| {
        std::fs::write(staged, bytes).map_err(Error::Io)
    })
}

fn stage_and_publish(
    path: &Path,
    extension: &str,
    write: impl FnOnce(&Path) -> Result<(), Error>,
) -> Result<(), Error> {
    let staged = reserve_unique_sibling_temp_path(path, extension)?;
    with_temporary_artifacts(vec![staged.clone()], || {
        write(&staged)?;
        publish_staged(&staged, path)
    })
}

pub(crate) fn with_temporary_artifacts<T>(
    paths: Vec<PathBuf>,
    work: impl FnOnce() -> Result<T, Error>,
) -> Result<T, Error> {
    let mut artifacts = TemporaryArtifacts { paths };
    let result = work();
    let cleanup = artifacts.cleanup();
    crate::render::backend::finish_scope(result, cleanup, "temporary render artifacts")
}

struct TemporaryArtifacts {
    paths: Vec<PathBuf>,
}

impl TemporaryArtifacts {
    fn cleanup(&mut self) -> Result<(), Error> {
        let mut failures = Vec::new();
        self.paths.retain(|path| match remove_if_present(path) {
            Ok(()) => false,
            Err(error) => {
                failures.push(format!("{}: {error}", path.display()));
                true
            }
        });
        cleanup_result(&failures, "remove temporary artifacts")
    }
}

impl Drop for TemporaryArtifacts {
    fn drop(&mut self) {
        for path in &self.paths {
            let _ = remove_if_present(path);
        }
    }
}

fn remove_if_present(path: &Path) -> std::io::Result<()> {
    #[cfg(test)]
    {
        REMOVE_ATTEMPTS.with(|attempts| attempts.borrow_mut().push(path.to_path_buf()));
        let should_fail = REMOVE_FAILURES.with(|failures| {
            let mut failures = failures.borrow_mut();
            let Some((_, remaining)) = failures
                .iter_mut()
                .find(|(candidate, remaining)| candidate == path && *remaining > 0)
            else {
                return false;
            };
            *remaining -= 1;
            true
        });
        if should_fail {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "injected temporary artifact removal failure",
            ));
        }
    }
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn cleanup_result(failures: &[String], action: &str) -> Result<(), Error> {
    if failures.is_empty() {
        Ok(())
    } else {
        Err(Error::Render(format!(
            "failed to {action}: {}",
            failures.join("; ")
        )))
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
        fail_removals(path, 1);
    }

    fn fail_removals(path: &Path, count: usize) {
        REMOVE_FAILURES.with(|failures| failures.borrow_mut().push((path.to_path_buf(), count)));
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
    fn reservation_retries_collisions_and_keeps_sibling_placement() {
        let directory = test_directory("reservation-collision");
        let collision = directory.join("collision.svg");
        let replacement = directory.join("replacement.svg");
        std::fs::write(&collision, "occupied").unwrap();
        let mut candidates = [collision.clone(), replacement.clone()].into_iter();

        let reserved = reserve_unique_path(|| Ok(candidates.next().unwrap())).unwrap();

        assert_eq!(reserved, replacement);
        assert_eq!(reserved.parent(), Some(directory.as_path()));
        assert_eq!(std::fs::read_to_string(collision).unwrap(), "occupied");
        assert!(reserved.is_file());
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn reservation_rejects_path_without_file_name() {
        let error = reserve_unique_sibling_temp_path(Path::new("/"), "svg").unwrap_err();
        assert!(error.to_string().contains("has no file name"));
    }

    #[test]
    fn reservation_propagates_non_collision_io_failure() {
        let directory = test_directory("reservation-io-failure");
        let output = directory.join("missing").join("plot.svg");

        let error = reserve_unique_sibling_temp_path(&output, "svg").unwrap_err();

        assert!(matches!(error, Error::Io(_)));
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn staged_write_failure_preserves_destination_and_removes_partial_file() {
        reset_removal_evidence();
        let directory = test_directory("write-failure");
        let output = directory.join("plot.svg");
        std::fs::write(&output, "existing artifact").unwrap();
        let staged_path = std::cell::RefCell::new(None);

        let result = stage_and_publish(&output, "svg", |staged| {
            staged_path.replace(Some(staged.to_path_buf()));
            std::fs::write(staged, "partial artifact")?;
            fail_next_removal(staged);
            Err(std::io::Error::new(std::io::ErrorKind::WriteZero, "staged write failed").into())
        });
        let staged = staged_path.into_inner().unwrap();

        let error = result.unwrap_err().to_string();
        assert!(error.contains("staged write failed"));
        assert!(error.contains("injected temporary artifact removal failure"));
        assert_eq!(
            std::fs::read_to_string(output).unwrap(),
            "existing artifact"
        );
        assert_eq!(removal_attempts(&staged), 2);
        assert!(!staged.exists());
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn publication_and_cleanup_failures_preserve_destination_and_retry_cleanup() {
        reset_removal_evidence();
        let directory = test_directory("publication-cleanup-failure");
        let output = directory.join("plot.svg");
        std::fs::create_dir(&output).unwrap();
        std::fs::write(output.join("existing"), "existing artifact").unwrap();
        let staged = reserve_unique_sibling_temp_path(&output, "svg").unwrap();
        std::fs::write(&staged, "replacement artifact").unwrap();
        fail_next_removal(&staged);

        let result =
            with_temporary_artifacts(vec![staged.clone()], || publish_staged(&staged, &output));
        let error = result.unwrap_err().to_string();

        assert!(error.contains("additionally failed to close temporary render artifacts"));
        assert_eq!(
            std::fs::read_to_string(output.join("existing")).unwrap(),
            "existing artifact"
        );
        assert_eq!(removal_attempts(&staged), 2);
        assert!(!staged.exists());
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn cleanup_aggregates_in_path_order_and_retains_the_primary_error() {
        reset_removal_evidence();
        let directory = test_directory("cleanup-aggregation");
        let first = directory.join("first");
        let removable = directory.join("removable");
        let second = directory.join("second");
        std::fs::write(&first, "temporary").unwrap();
        std::fs::write(&removable, "temporary").unwrap();
        std::fs::write(&second, "temporary").unwrap();
        fail_next_removal(&first);
        fail_next_removal(&second);

        let result: Result<(), Error> = with_temporary_artifacts(
            vec![first.clone(), removable.clone(), second.clone()],
            || Err(Error::Render("primary publication failure".into())),
        );
        let error = result.unwrap_err().to_string();

        assert!(error.contains("primary publication failure"));
        assert!(
            error.find(&first.display().to_string()).unwrap()
                < error.find(&second.display().to_string()).unwrap()
        );
        assert!(!first.exists() && !removable.exists() && !second.exists());
        assert_eq!(removal_attempts(&first), 2);
        assert_eq!(removal_attempts(&removable), 1);
        assert_eq!(removal_attempts(&second), 2);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn drop_removes_tracked_staging_file_during_unwind() {
        let directory = test_directory("drop-cleanup");
        let output = directory.join("plot.svg");
        let staged = reserve_unique_sibling_temp_path(&output, "svg").unwrap();
        let observed = staged.clone();

        let unwind = std::panic::catch_unwind(|| {
            let _artifacts = TemporaryArtifacts {
                paths: vec![staged],
            };
            panic!("simulate transaction unwind");
        });

        assert!(unwind.is_err());
        assert!(!observed.exists());
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn drop_suppresses_cleanup_failure_during_unwind() {
        reset_removal_evidence();
        let directory = test_directory("drop-cleanup-failure");
        let output = directory.join("plot.svg");
        let staged = reserve_unique_sibling_temp_path(&output, "svg").unwrap();
        let observed = staged.clone();
        fail_removals(&staged, 1);

        let unwind = std::panic::catch_unwind(|| {
            let _artifacts = TemporaryArtifacts {
                paths: vec![staged],
            };
            panic!("simulate transaction unwind");
        });

        assert!(unwind.is_err());
        assert_eq!(removal_attempts(&observed), 1);
        assert!(observed.exists());
        reset_removal_evidence();
        std::fs::remove_file(observed).unwrap();
        std::fs::remove_dir_all(directory).unwrap();
    }
}
