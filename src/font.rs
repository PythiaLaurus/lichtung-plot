//! Canonical system-font preparation and current rendering defaults.

pub(crate) mod canonical;
mod unicode16;

#[cfg(test)]
pub(crate) fn unicode16_source(name: &str) -> String {
    use sha2::{Digest, Sha256};

    let manifest = include_str!("../tests/fixtures/unicode16/sources.json");
    let prefix = format!("  \"{name}\": \"");
    let expected = manifest
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .and_then(|value| value.split_once('"').map(|(hash, _)| hash))
        .expect("source must be a pinned Unicode 16 member");
    let directory = std::env::var_os("LICHTUNG_UNICODE16_SOURCE_DIR")
        .expect("set LICHTUNG_UNICODE16_SOURCE_DIR to the original extracted Unicode 16 UCD");
    let path = std::path::Path::new(&directory).join(name);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("cannot read Unicode source {}: {error}", path.display()));
    assert_eq!(
        format!("{:x}", Sha256::digest(&bytes)),
        expected,
        "source checksum mismatch: {}",
        path.display()
    );
    // Verify original bytes before removing comments; keep section directives.
    String::from_utf8(bytes)
        .expect("Unicode source must be UTF-8")
        .lines()
        .filter_map(|line| {
            let body = line.split('#').next().unwrap().trim();
            (!body.is_empty()).then_some(body)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The literal name of the first named family in the built-in typography profile.
///
/// This string denotes that one family; it is not an operation-default sentinel.
pub const DEFAULT_FONT_FAMILY: &str = "Noto Sans";

/// CSS publication spelling of the built-in typography profile.
///
/// Built-in roles retain typed preferences directly, independently of this string.
pub const DEFAULT_FONT_FAMILY_STACK: &str =
    "Noto Sans, Noto Sans CJK SC, DejaVu Sans, Liberation Sans, Arial, sans-serif";
