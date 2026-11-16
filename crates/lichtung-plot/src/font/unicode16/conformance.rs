fn fixture(name: &str) -> String {
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/unicode16")
            .join(name),
    )
    .unwrap()
}

fn normalize(value: &str) -> String {
    super::nfc::normalize(value).unwrap()
}

#[test]
fn focused_nfc_and_casefold() {
    // Literal expectations cover ordering, blocking, Hangul, and mapping boundaries.
    for (input, expected) in [
        ("A\u{030A}", "\u{00C5}"),
        ("a\u{0315}\u{0300}", "\u{00E0}\u{0315}"),
        ("A\u{0305}\u{030A}", "A\u{0305}\u{030A}"),
        ("\u{1100}\u{1161}\u{11A8}", "\u{AC01}"),
        ("\u{0344}", "\u{0308}\u{0301}"),
    ] {
        assert_eq!(normalize(input), expected, "{input:?}");
    }
    for (input, expected) in [
        ("Stra\u{00DF}e", "strasse"),
        ("\u{FF21}\u{030A}", "\u{00E5}"),
        ("A\u{034F}\u{030A}", "\u{00E5}"),
        ("\u{FB03}", "ffi"),
        ("\u{03A3}\u{03C2}", "\u{03C3}\u{03C3}"),
    ] {
        assert_eq!(super::folded_for_test(input), expected, "{input:?}");
    }
}

#[test]
fn packaged_unicode_provenance() {
    use sha2::{Digest, Sha256};
    for row in fixture("SHA256SUMS").lines() {
        let (expected, path) = row.split_once("  ").unwrap();
        let data =
            std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap();
        assert_eq!(format!("{:x}", Sha256::digest(data)), expected, "{path}");
    }
}
