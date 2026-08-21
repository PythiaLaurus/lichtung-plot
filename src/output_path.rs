//! Render-output destination identity.

use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

use crate::Error;

enum UnresolvedComponent {
    Current,
    Parent,
    Normal(OsString),
}

/// Resolve an output path to the identity the filesystem will use.
///
/// Existing prefixes are canonicalized before unresolved components are
/// applied, so symlinks are followed before a subsequent `..` is interpreted.
pub(crate) fn destination_identity(path: &Path) -> Result<PathBuf, Error> {
    let mut cursor = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut unresolved = Vec::new();
    let mut resolved = loop {
        match std::fs::canonicalize(&cursor) {
            Ok(resolved) => break resolved,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let component = cursor.components().next_back().ok_or_else(|| {
                    Error::Data(format!(
                        "render output path {} cannot be resolved",
                        path.display()
                    ))
                })?;
                unresolved.push(match component {
                    Component::CurDir => UnresolvedComponent::Current,
                    Component::ParentDir => UnresolvedComponent::Parent,
                    Component::Normal(component) => {
                        UnresolvedComponent::Normal(component.to_os_string())
                    }
                    Component::Prefix(_) | Component::RootDir => {
                        return Err(Error::Data(format!(
                            "render output path {} has no resolvable ancestor",
                            path.display()
                        )))
                    }
                });
                if !cursor.pop() {
                    return Err(Error::Data(format!(
                        "render output path {} has no resolvable ancestor",
                        path.display()
                    )));
                }
            }
            Err(error) => return Err(error.into()),
        }
    };

    for component in unresolved.into_iter().rev() {
        match component {
            UnresolvedComponent::Current => {}
            UnresolvedComponent::Parent => {
                resolved.pop();
            }
            UnresolvedComponent::Normal(component) => resolved.push(component),
        }
    }
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::destination_identity;

    static PATH_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_directory(label: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "lichtung-output-path-{}-{}-{label}",
            std::process::id(),
            PATH_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        path
    }

    #[test]
    fn missing_components_are_normalized_after_the_existing_ancestor() {
        let dir = test_directory("missing-components");
        let first = dir.join("missing/nested/../plot.svg");
        let second = dir.join("missing/plot.svg");

        assert_eq!(
            destination_identity(&first).unwrap(),
            destination_identity(&second).unwrap()
        );
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn parent_after_symlink_is_applied_to_the_symlink_target() {
        use std::os::unix::fs::symlink;

        let dir = test_directory("symlink-parent");
        let target_parent = dir.join("target");
        let target_child = target_parent.join("child");
        std::fs::create_dir_all(&target_child).unwrap();
        let link = dir.join("link");
        symlink(&target_child, &link).unwrap();

        assert_eq!(
            destination_identity(&link.join("../plot.svg")).unwrap(),
            destination_identity(&target_parent.join("plot.svg")).unwrap()
        );
        std::fs::remove_dir_all(dir).unwrap();
    }
}
