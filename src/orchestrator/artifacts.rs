//! Atomic render-artifact staging, publication, and rollback.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::Error;

#[cfg(any(feature = "png", test))]
use crate::output_path::destination_identity;

pub(super) static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[cfg(any(feature = "png", test))]
pub(crate) fn paths_resolve_to_same_destination(
    first: &Path,
    second: &Path,
) -> Result<bool, Error> {
    Ok(destination_identity(first)? == destination_identity(second)?)
}

pub(crate) fn reserve_unique_sibling_temp_path(
    path: &Path,
    extension: &str,
) -> Result<PathBuf, Error> {
    loop {
        let candidate = unique_sibling_temp_path(path, extension)?;
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

#[cfg(feature = "png")]
pub(crate) fn reserve_staging_pair(
    first_target: &Path,
    first_extension: &str,
    second_target: &Path,
    second_extension: &str,
) -> Result<(PathBuf, PathBuf), Error> {
    let first = reserve_unique_sibling_temp_path(first_target, first_extension)?;
    match reserve_unique_sibling_temp_path(second_target, second_extension) {
        Ok(second) => Ok((first, second)),
        Err(error) => crate::render::backend::finish_scope(
            Err(error),
            remove_if_present(&first).map_err(Error::Io),
            "partial staging reservation",
        ),
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
        publish_staged_pair([(staged, final_path)])
    }
}

#[cfg(any(feature = "png", windows))]
pub(crate) fn publish_staged_pair<const N: usize>(paths: [(&Path, &Path); N]) -> Result<(), Error> {
    for (_, final_path) in paths {
        if final_path.exists() && !std::fs::metadata(final_path)?.is_file() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("render destination {} is not a file", final_path.display()),
            )
            .into());
        }
    }
    let mut backups = Vec::with_capacity(N);
    for (_, final_path) in paths {
        if final_path.exists() {
            let backup = unique_sibling_temp_path(final_path, "backup")?;
            if let Err(error) = std::fs::rename(final_path, &backup) {
                let restored = restore_backups(&backups);
                return crate::render::backend::finish_scope(
                    Err(error.into()),
                    restored,
                    "artifact publication rollback",
                );
            }
            backups.push((backup, final_path.to_path_buf()));
        }
    }

    let mut published = Vec::with_capacity(N);
    for (staged, final_path) in paths {
        if let Err(error) = std::fs::rename(staged, final_path) {
            let cleanup = rollback_publication(&published, &backups);
            return crate::render::backend::finish_scope(
                Err(error.into()),
                cleanup,
                "artifact publication rollback",
            );
        }
        published.push(final_path.to_path_buf());
    }

    finish_committed_publication(&backups, remove_if_present);
    Ok(())
}

#[cfg(any(feature = "png", windows))]
pub(super) fn finish_committed_publication(
    backups: &[(PathBuf, PathBuf)],
    mut remove: impl FnMut(&Path) -> std::io::Result<()>,
) {
    for (backup, _) in backups {
        let _ = remove(backup);
    }
}

#[cfg(any(feature = "png", windows))]
fn rollback_publication(
    published: &[PathBuf],
    backups: &[(PathBuf, PathBuf)],
) -> Result<(), Error> {
    rollback_publication_with(published, backups, remove_if_present, |from, to| {
        std::fs::rename(from, to)
    })
}

#[cfg(any(feature = "png", windows))]
fn rollback_publication_with(
    published: &[PathBuf],
    backups: &[(PathBuf, PathBuf)],
    mut remove: impl FnMut(&Path) -> std::io::Result<()>,
    mut restore: impl FnMut(&Path, &Path) -> std::io::Result<()>,
) -> Result<(), Error> {
    let mut failures = Vec::new();
    for path in published {
        if let Err(error) = remove(path) {
            failures.push(error.to_string());
        }
    }
    for (backup, final_path) in backups.iter().rev() {
        if let Err(error) = restore(backup, final_path) {
            failures.push(format!(
                "restore {} to {}: {error}",
                backup.display(),
                final_path.display()
            ));
        }
    }
    cleanup_result(&failures, "artifact publication rollback")
}

#[cfg(any(feature = "png", windows))]
fn restore_backups(backups: &[(PathBuf, PathBuf)]) -> Result<(), Error> {
    let mut failures = Vec::new();
    for (backup, final_path) in backups.iter().rev() {
        if let Err(error) = std::fs::rename(backup, final_path) {
            failures.push(error.to_string());
        }
    }
    cleanup_result(&failures, "restore previous artifacts")
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

#[cfg(all(test, feature = "png"))]
mod tests {
    use super::*;

    #[test]
    fn rollback_reports_both_published_cleanup_and_backup_restore_failures() {
        let published = vec![PathBuf::from("new.svg")];
        let backups = vec![(PathBuf::from("old.backup"), PathBuf::from("old.svg"))];

        let error = rollback_publication_with(
            &published,
            &backups,
            |_| {
                Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "cannot remove partial publication",
                ))
            },
            |_, _| {
                Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "cannot restore backup",
                ))
            },
        )
        .unwrap_err()
        .to_string();

        assert!(error.contains("cannot remove partial publication"));
        assert!(error.contains("restore old.backup to old.svg: cannot restore backup"));
    }
}
