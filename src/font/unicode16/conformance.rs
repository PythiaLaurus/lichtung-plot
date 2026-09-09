fn fixture(name: &str) -> String {
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/unicode16")
            .join(name),
    )
    .unwrap()
}

fn decode(value: &str) -> String {
    value
        .split_whitespace()
        .map(|cp| char::from_u32(u32::from_str_radix(cp, 16).unwrap()).unwrap())
        .collect()
}

fn normalize(value: &str) -> String {
    let resources = crate::font::canonical::OperationFontResources::new();
    super::nfc::normalize(value, &mut |bytes, work| {
        resources.claim_bytes(bytes)?;
        resources.claim_work(work)
    })
    .unwrap()
}

#[test]
fn official_nfc() {
    let mut count = 0;
    let mut failures = Vec::new();
    let mut part1 = false;
    let mut listed = std::collections::HashSet::new();
    for (line, row) in fixture("NormalizationTest.txt").lines().enumerate() {
        if row.starts_with('@') {
            part1 = row == "@Part1";
            continue;
        }
        let columns: Vec<_> = row.split(';').take(5).map(decode).collect();
        if part1 {
            assert_eq!(columns[0].chars().count(), 1);
            listed.insert(columns[0].chars().next().unwrap());
        }
        for (input, expected) in [(0, 1), (1, 1), (2, 1), (3, 3), (4, 3)] {
            if normalize(&columns[input]) != columns[expected] {
                failures.push((line + 1, input, row.to_owned()));
            }
        }
        count += 1;
    }
    assert!(
        failures.is_empty(),
        "{} failures: {:?}",
        failures.len(),
        &failures[..failures.len().min(12)]
    );
    assert_eq!(count, 19965);
    // NormalizationTest's second invariant: unlisted Part1 scalars are NFC.
    for scalar in (0..=0x0010_FFFF).filter_map(char::from_u32) {
        if !listed.contains(&scalar) {
            let input = scalar.to_string();
            assert_eq!(normalize(&input), input);
        }
    }
}

#[test]
fn official_nfkc_casefold() {
    let mut count = 0;
    let mut mapped = std::collections::HashSet::new();
    for row in fixture("DerivedNormalizationProps.txt").lines() {
        let fields: Vec<_> = row.split(';').map(str::trim).collect();
        if fields[1] != "NFKC_CF" {
            continue;
        }
        let range: Vec<_> = fields[0]
            .split("..")
            .map(|cp| u32::from_str_radix(cp, 16).unwrap())
            .collect();
        let expected = decode(fields[2]);
        for cp in range[0]..=*range.last().unwrap() {
            let input = char::from_u32(cp).unwrap().to_string();
            assert_eq!(super::folded_for_test(&input), expected, "U+{cp:04X}");
            count += 1;
            mapped.insert(cp);
        }
    }
    assert_eq!(count, 10554);
    // The official @missing value for NFKC_CF is <code point>.
    for scalar in (0..=0x0010_FFFF).filter_map(char::from_u32) {
        if !mapped.contains(&(scalar as u32)) {
            let input = scalar.to_string();
            assert_eq!(super::folded_for_test(&input), input);
        }
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
