//! Explicit, offline regeneration utility within the existing Rust test suite.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use sha2::{Digest, Sha256};

const ARCHIVE_SHA256: &str = "c86dd81f2b14a43b0cc064aa5f89aa7241386801e35c59c7984e579832634eb2";
const HEADER: &str = "// Generated from Unicode 16.0.0; see tests/fixtures/unicode16/README.md.\n";
const SOURCES: &[&str] = &[
    "UnicodeData.txt",
    "DerivedNormalizationProps.txt",
    "LineBreak.txt",
    "EastAsianWidth.txt",
    "emoji/emoji-data.txt",
    "auxiliary/GraphemeBreakTest.txt",
    "auxiliary/LineBreakTest.txt",
    "NormalizationTest.txt",
    "BidiTest.txt",
    "BidiCharacterTest.txt",
    "ReadMe.txt",
];

// This deliberately accepts only the checked-in manifest's canonical JSON
// subset: one unescaped, allowlisted filename and lowercase SHA-256 per line.
fn source_hashes(manifest: &str) -> Result<BTreeMap<&str, &str>, &'static str> {
    let lines: Vec<_> = manifest.lines().collect();
    if lines.len() != SOURCES.len() + 2 || lines[0] != "{" || lines[lines.len() - 1] != "}" {
        return Err("invalid source manifest framing");
    }
    let mut hashes = BTreeMap::new();
    for (index, line) in lines[1..lines.len() - 1].iter().enumerate() {
        let suffix = if index + 1 == SOURCES.len() {
            "\""
        } else {
            "\","
        };
        let body = line
            .strip_prefix("  \"")
            .and_then(|line| line.strip_suffix(suffix))
            .ok_or("invalid source manifest entry")?;
        let (name, hash) = body
            .split_once("\": \"")
            .ok_or("invalid source manifest separator")?;
        if !SOURCES.contains(&name)
            || hash.len() != 64
            || !hash
                .bytes()
                .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
        {
            return Err("invalid source name or checksum");
        }
        if hashes.insert(name, hash).is_some() {
            return Err("duplicate source name");
        }
    }
    if hashes.len() != SOURCES.len() {
        return Err("missing source member");
    }
    Ok(hashes)
}

fn records(source: &str) -> impl Iterator<Item = Vec<&str>> {
    source.lines().filter_map(|line| {
        let body = line.split('#').next().unwrap().trim();
        (!body.is_empty() && !body.starts_with('@'))
            .then(|| body.split(';').map(str::trim).collect())
    })
}

fn bounds(value: &str) -> (u32, u32) {
    let parts: Vec<_> = value.split("..").collect();
    assert!((1..=2).contains(&parts.len()), "invalid range {value}");
    let first = u32::from_str_radix(parts[0], 16).unwrap();
    let last = u32::from_str_radix(parts[parts.len() - 1], 16).unwrap();
    assert!(
        (first..=0x0010_FFFF).contains(&last),
        "invalid range {value}"
    );
    (first, last)
}

fn rust_string(value: &str) -> String {
    let mut output = String::from("\"");
    for codepoint in value.split_whitespace() {
        let codepoint = u32::from_str_radix(codepoint, 16).unwrap();
        assert!(char::from_u32(codepoint).is_some());
        write!(output, "\\u{{{codepoint:X}}}").unwrap();
    }
    output.push('"');
    output
}

fn ranges<T: Eq>(mut input: Vec<(u32, u32, T)>) -> Vec<(u32, u32, T)> {
    input.sort_by_key(|row| row.0);
    let mut output: Vec<(u32, u32, T)> = Vec::new();
    for (first, last, value) in input {
        if let Some(previous) = output.last_mut() {
            assert!(previous.1 < first, "overlapping Unicode ranges");
            if previous.1 + 1 == first && previous.2 == value {
                previous.1 = last;
                continue;
            }
        }
        output.push((first, last, value));
    }
    output
}

fn table(name: &str, element: &str, rows: impl IntoIterator<Item = String>) -> String {
    let mut output = format!("pub(super) static {name}: &[{element}] = &[\n");
    for row in rows {
        writeln!(output, "    ({row}),").unwrap();
    }
    output.push_str("];\n\n");
    output
}

fn unicode_data(source: &str) -> Vec<(u32, u32, Vec<&str>)> {
    let mut output = Vec::new();
    let mut pending: Option<(u32, Vec<&str>)> = None;
    for fields in records(source) {
        assert_eq!(fields.len(), 15);
        let codepoint = u32::from_str_radix(fields[0], 16).unwrap();
        if fields[1].ends_with(", First>") {
            assert!(pending.replace((codepoint, fields)).is_none());
            continue;
        }
        let first = if fields[1].ends_with(", Last>") {
            let (first, initial) = pending.take().expect("Last without First");
            assert_eq!(&initial[2..], &fields[2..]);
            assert!(first < codepoint);
            first
        } else {
            assert!(pending.is_none(), "unterminated UnicodeData range");
            codepoint
        };
        output.push((first, codepoint, fields));
    }
    assert!(pending.is_none());
    output
}

fn normalization_tables(data: &[(u32, u32, Vec<&str>)], properties: &str) -> (String, String) {
    let mut excluded = BTreeSet::new();
    let mut fold = Vec::new();
    for row in records(properties) {
        let (first, last) = bounds(row[0]);
        match row[1] {
            "Full_Composition_Exclusion" => excluded.extend(first..=last),
            "NFKC_CF" => fold.push((first, last, row[2])),
            _ => (),
        }
    }
    fold.sort_by_key(|row| (row.0, row.1));
    let casefold = HEADER.to_owned()
        + &table(
            "NFKC_CASEFOLD",
            "(u32, u32, &str)",
            fold.iter().map(|(first, last, value)| {
                format!("0x{first:X}, 0x{last:X}, {}", rust_string(value))
            }),
        );
    let mut combining = Vec::new();
    let mut decomposition = Vec::new();
    let mut composition = Vec::new();
    for (first, last, fields) in data {
        let class: u8 = fields[3].parse().unwrap();
        if class != 0 {
            combining.push((*first, *last, class));
        }
        if fields[5].is_empty() || fields[5].starts_with('<') {
            continue;
        }
        assert_eq!(first, last);
        decomposition.push((*first, fields[5]));
        let parts: Vec<_> = fields[5].split_whitespace().collect();
        if parts.len() == 2 && !excluded.contains(first) {
            composition.push((
                u32::from_str_radix(parts[0], 16).unwrap(),
                u32::from_str_radix(parts[1], 16).unwrap(),
                *first,
            ));
        }
    }
    decomposition.sort_by_key(|row| row.0);
    composition.sort_unstable();
    let nfc = HEADER.to_owned()
        + &table(
            "CANONICAL_COMBINING_CLASS",
            "(u32, u32, u8)",
            ranges(combining)
                .iter()
                .map(|(first, last, class)| format!("0x{first:X}, 0x{last:X}, {class}")),
        )
        + &table(
            "CANONICAL_DECOMPOSITION",
            "(u32, &str)",
            decomposition
                .iter()
                .map(|(codepoint, value)| format!("0x{codepoint:X}, {}", rust_string(value))),
        )
        + &table(
            "CANONICAL_COMPOSITION",
            "(u32, u32, u32)",
            composition.iter().map(|(first, second, composed)| {
                format!("0x{first:X}, 0x{second:X}, 0x{composed:X}")
            }),
        );
    (casefold, nfc)
}

fn linebreak_table(data: &[(u32, u32, Vec<&str>)], sources: &BTreeMap<&str, String>) -> String {
    let linebreaks = records(&sources["LineBreak.txt"])
        .map(|row| {
            let (first, last) = bounds(row[0]);
            (first, last, row[1])
        })
        .collect();
    let mut output = HEADER.to_owned()
        + "#![allow(clippy::unreadable_literal)]\nuse super::Class;\n\n"
        + &table(
            "LINE_BREAK",
            "(u32, u32, Class)",
            ranges(linebreaks)
                .iter()
                .map(|(first, last, class)| format!("0x{first:X}, 0x{last:X}, Class::{class}")),
        );
    let mut assigned = BTreeSet::new();
    let mut marks = Vec::new();
    let mut initial = Vec::new();
    let mut final_punctuation = Vec::new();
    for (first, last, fields) in data {
        assigned.extend(*first..=*last);
        match fields[2] {
            "Mn" | "Mc" => marks.push((*first, *last, ())),
            "Pi" => initial.push((*first, *last, ())),
            "Pf" => final_punctuation.push((*first, *last, ())),
            _ => (),
        }
    }
    let mut east_asian = Vec::new();
    for row in records(&sources["EastAsianWidth.txt"]) {
        if matches!(row[1], "F" | "W" | "H") {
            let (first, last) = bounds(row[0]);
            east_asian.push((first, last, ()));
        }
    }
    let mut pictographic = Vec::new();
    for row in records(&sources["emoji/emoji-data.txt"]) {
        if row[1] == "Extended_Pictographic" {
            let (first, last) = bounds(row[0]);
            pictographic.extend(
                (first..=last)
                    .filter(|cp| !assigned.contains(cp))
                    .map(|cp| (cp, cp, ())),
            );
        }
    }
    for (name, values) in [
        ("MARK", marks),
        ("INITIAL_PUNCTUATION", initial),
        ("FINAL_PUNCTUATION", final_punctuation),
        ("EAST_ASIAN", east_asian),
        ("UNASSIGNED_EXTENDED_PICTOGRAPHIC", pictographic),
    ] {
        output.push_str(&table(
            name,
            "(u32, u32)",
            ranges(values)
                .iter()
                .map(|(first, last, ())| format!("0x{first:X}, 0x{last:X}")),
        ));
    }
    output
}

fn formatted(source: &str, root: &Path) -> String {
    let mut child = Command::new("rustfmt")
        .args(["--emit", "stdout", "--edition", "2021"])
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("run the repository toolchain's rustfmt");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(source.as_bytes())
        .unwrap();
    let result = child.wait_with_output().unwrap();
    assert!(
        result.status.success(),
        "rustfmt: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    String::from_utf8(result.stdout).unwrap()
}

#[test]
#[ignore = "requires LICHTUNG_UNICODE16_SOURCE_DIR pointing to an explicit local UCD directory"]
fn regenerate_unicode16() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let directory = PathBuf::from(
        std::env::var_os("LICHTUNG_UNICODE16_SOURCE_DIR")
            .expect("set an explicit local UCD directory"),
    );
    assert!(
        directory.is_dir(),
        "source must be an extracted UCD directory"
    );
    let write = match std::env::var_os("LICHTUNG_UNICODE16_WRITE") {
        None => false,
        Some(value) if value == "1" => true,
        Some(_) => panic!("LICHTUNG_UNICODE16_WRITE must be unset (check only) or exactly 1"),
    };
    if let Some(archive) = std::env::var_os("LICHTUNG_UNICODE16_ARCHIVE") {
        assert_eq!(
            format!("{:x}", Sha256::digest(std::fs::read(archive).unwrap())),
            ARCHIVE_SHA256,
            "archive checksum mismatch"
        );
    }
    let manifest =
        std::fs::read_to_string(root.join("tests/fixtures/unicode16/sources.json")).unwrap();
    let hashes = source_hashes(&manifest).expect("valid pinned source manifest");
    let mut sources = BTreeMap::new();
    // Validate every original byte before generating or writing any output.
    for name in SOURCES {
        let bytes = std::fs::read(directory.join(name)).unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(&bytes)),
            hashes[name],
            "source checksum mismatch: {name}"
        );
        sources.insert(*name, String::from_utf8(bytes).unwrap());
    }
    let data = unicode_data(&sources["UnicodeData.txt"]);
    let (casefold, nfc) = normalization_tables(&data, &sources["DerivedNormalizationProps.txt"]);
    let mut outputs = BTreeMap::new();
    for (path, source) in [
        ("src/font/unicode16/casefold_data.rs", casefold),
        ("src/font/unicode16/nfc_data.rs", nfc),
        (
            "src/layout/unicode16/data.rs",
            linebreak_table(&data, &sources),
        ),
    ] {
        outputs.insert(path.to_owned(), formatted(&source, root));
    }
    for name in SOURCES {
        if name.ends_with("Test.txt") || *name == "DerivedNormalizationProps.txt" {
            let mut compact = String::new();
            for line in sources[name].lines() {
                let body = line.split('#').next().unwrap().trim();
                if !body.is_empty() {
                    writeln!(compact, "{body}").unwrap();
                }
            }
            let leaf = Path::new(name).file_name().unwrap().to_str().unwrap();
            outputs.insert(format!("tests/fixtures/unicode16/{leaf}"), compact);
        }
    }
    outputs.insert(
        "tests/fixtures/unicode16/ReadMe.txt".to_owned(),
        sources["ReadMe.txt"].clone(),
    );
    let mut sums = String::new();
    for (path, content) in &outputs {
        writeln!(sums, "{:x}  {path}", Sha256::digest(content.as_bytes())).unwrap();
    }
    outputs.insert("tests/fixtures/unicode16/SHA256SUMS".to_owned(), sums);
    for (path, content) in &outputs {
        let destination = root.join(path);
        if write {
            std::fs::write(&destination, content).unwrap();
        } else {
            let retained = std::fs::read(&destination).unwrap();
            assert!(
                retained == content.as_bytes(),
                "regeneration differs: {path}; retained SHA-256 {:x}, generated {:x}",
                Sha256::digest(&retained),
                Sha256::digest(content.as_bytes())
            );
        }
        eprintln!("{:x}  {path}", Sha256::digest(content.as_bytes()));
    }
    assert_eq!(outputs.len(), 11);
    eprintln!("{} validated source members; {} {} outputs (three tables, six fixtures, ReadMe, SHA256SUMS)", sources.len(), outputs.len(), if write { "regenerated" } else { "byte-identical" });
}

#[test]
fn source_manifest_rejects_unknown_duplicate_and_malformed_entries() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest =
        std::fs::read_to_string(root.join("tests/fixtures/unicode16/sources.json")).unwrap();
    assert_eq!(source_hashes(&manifest).unwrap().len(), SOURCES.len());
    for invalid in [
        manifest.replace("UnicodeData.txt", "../UnicodeData.txt"),
        manifest.replace("ReadMe.txt", "UnicodeData.txt"),
        manifest.replace("ff58e582", "FF58E582"),
        manifest.replace("\": \"", "\": "),
        manifest.replace("{\n", "{\n\n"),
    ] {
        assert!(source_hashes(&invalid).is_err());
    }
    let readme = std::fs::read_to_string(root.join("tests/fixtures/unicode16/README.md")).unwrap();
    assert!(readme.contains(ARCHIVE_SHA256));
}
