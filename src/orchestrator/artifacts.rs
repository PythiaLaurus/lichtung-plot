//! Atomic render-artifact staging, publication, and rollback.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::Error;

pub(super) static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

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
    if !final_path.exists() {
        std::fs::rename(staged, final_path)?;
        return Ok(());
    }
    if !std::fs::metadata(final_path)?.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("render destination {} is not a file", final_path.display()),
        )
        .into());
    }

    let backup = unique_sibling_temp_path(final_path, "backup")?;
    std::fs::rename(final_path, &backup)?;
    if let Err(error) = std::fs::rename(staged, final_path) {
        return crate::render::backend::finish_scope(
            Err(error.into()),
            std::fs::rename(&backup, final_path).map_err(Error::Io),
            "artifact publication rollback",
        );
    }
    let _ = remove_if_present(&backup);
    Ok(())
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
