use unicode_bidi::{BidiInfo, Level};
use unicode_segmentation::UnicodeSegmentation;

use crate::font::unicode16_source as fixture;

fn check_bidi(
    text: &str,
    direction: Option<Level>,
    paragraph: Option<u8>,
    expected_levels: &str,
    expected_order: &str,
) {
    let info = BidiInfo::new(text, direction);
    assert_eq!(info.paragraphs.len(), 1);
    let para = &info.paragraphs[0];
    if let Some(expected) = paragraph {
        assert_eq!(para.level.number(), expected);
    }
    let levels = info.reordered_levels_per_char(para, para.range.clone());
    let expected: Vec<_> = expected_levels.split_whitespace().collect();
    assert_eq!(levels.len(), expected.len());
    for (actual, expected) in levels.iter().zip(&expected) {
        if *expected != "x" {
            assert_eq!(actual.number(), expected.parse::<u8>().unwrap(), "{text:?}");
        }
    }
    // X9 controls have no expected level or position. Remove them before L2;
    // retain original scalar indices for comparison with the official fixture.
    let retained: Vec<_> = levels
        .iter()
        .enumerate()
        .filter(|(index, _)| expected[*index] != "x")
        .collect();
    let order = BidiInfo::reorder_visual(
        &retained
            .iter()
            .map(|(_, level)| **level)
            .collect::<Vec<_>>(),
    );
    let actual: Vec<_> = order.iter().map(|index| retained[*index].0).collect();
    let expected: Vec<usize> = expected_order
        .split_whitespace()
        .map(|v| v.parse().unwrap())
        .collect();
    assert_eq!(actual, expected, "{text:?}");
}

#[test]
#[ignore = "requires LICHTUNG_UNICODE16_SOURCE_DIR with hash-verified original Unicode 16 sources"]
fn official_bidi_characters() {
    let mut count = 0;
    for (line, row) in fixture("BidiCharacterTest.txt").lines().enumerate() {
        let fields: Vec<_> = row.split(';').map(str::trim).collect();
        let text: String = fields[0]
            .split_whitespace()
            .map(|cp| char::from_u32(u32::from_str_radix(cp, 16).unwrap()).unwrap())
            .collect();
        let direction = match fields[1] {
            "0" => Some(Level::ltr()),
            "1" => Some(Level::rtl()),
            "2" => None,
            _ => panic!("invalid direction"),
        };
        let result = std::panic::catch_unwind(|| {
            check_bidi(
                &text,
                direction,
                Some(fields[2].parse().unwrap()),
                fields[3],
                fields[4],
            );
        });
        assert!(result.is_ok(), "BidiCharacterTest line {}: {row}", line + 1);
        count += 1;
    }
    assert_eq!(count, 91707);
}

#[test]
#[ignore = "requires LICHTUNG_UNICODE16_SOURCE_DIR with hash-verified original Unicode 16 sources"]
fn official_bidi_classes() {
    let source = fixture("BidiTest.txt");
    let mut levels = "";
    let mut order = "";
    let mut rows = 0;
    let mut runs = 0;
    for (line, row) in source.lines().enumerate() {
        if let Some(value) = row.strip_prefix("@Levels:") {
            levels = value;
            continue;
        }
        if let Some(value) = row.strip_prefix("@Reorder:") {
            order = value;
            continue;
        }
        let (classes, mask) = row.split_once(';').unwrap();
        let mask = u8::from_str_radix(mask.trim(), 16).unwrap();
        // Representatives have exactly the requested bidi class and no brackets.
        let text: String = classes
            .split_whitespace()
            .map(|class| match class {
                "L" => 'a',
                "R" => '\u{05D0}',
                "AL" => '\u{0627}',
                "EN" => '0',
                "ES" => '+',
                "ET" => '$',
                "AN" => '\u{0660}',
                "CS" => ',',
                "NSM" => '\u{0300}',
                "BN" => '\u{00AD}',
                "B" => '\u{2029}',
                "S" => '\t',
                "WS" => ' ',
                "ON" => '!',
                "LRE" => '\u{202A}',
                "RLE" => '\u{202B}',
                "PDF" => '\u{202C}',
                "LRO" => '\u{202D}',
                "RLO" => '\u{202E}',
                "LRI" => '\u{2066}',
                "RLI" => '\u{2067}',
                "FSI" => '\u{2068}',
                "PDI" => '\u{2069}',
                _ => panic!("invalid bidi class"),
            })
            .collect();
        for (bit, direction) in [(1, None), (2, Some(Level::ltr())), (4, Some(Level::rtl()))] {
            if mask & bit == 0 {
                continue;
            }
            let result =
                std::panic::catch_unwind(|| check_bidi(&text, direction, None, levels, order));
            assert!(result.is_ok(), "BidiTest line {}: {row}", line + 1);
            runs += 1;
        }
        rows += 1;
    }
    assert_eq!(rows, 490_846);
    assert_eq!(runs, 770_241);
    eprintln!("BidiTest: {rows} rows, {runs} directional runs");
}

#[test]
fn focused_bidi_and_graphemes() {
    check_bidi("a\u{05D0}\u{05D1}b", None, Some(0), "0 1 1 0", "0 2 1 3");
    check_bidi("\u{05D0}12", None, Some(1), "1 2 2", "1 2 0");
    check_bidi(
        "a\u{202E}bc\u{202C}d",
        None,
        Some(0),
        "0 x 1 1 x 0",
        "0 3 2 5",
    );
    check_bidi(
        "a\u{2067}\u{05D0}\u{05D1}\u{2069}b",
        None,
        Some(0),
        "0 0 1 1 0 0",
        "0 1 3 2 4 5",
    );
    for (text, expected) in [
        ("a\u{0301}b", vec![0, 3, 4]),
        ("\r\na", vec![0, 2, 3]),
        ("\u{1F469}\u{200D}\u{1F52C}", vec![0, 11]),
        ("\u{1F1E6}\u{1F1E7}\u{1F1E8}", vec![0, 8, 12]),
        ("\u{0915}\u{094D}\u{0915}", vec![0, 9]),
    ] {
        assert_eq!(
            text.grapheme_indices(true)
                .map(|(offset, _)| offset)
                .chain([text.len()])
                .collect::<Vec<_>>(),
            expected,
            "{text:?}"
        );
    }
}

#[test]
fn unicode_16_lb25_closing_punctuation_does_not_extend_digits() {
    use super::BreakOpportunity::{Allowed, Mandatory};
    // UAX #14 revision 53, LB25: NU (SY | IS)* x NU has no optional
    // CL/CP. Only the PO/PR suffix rules allow a closing punctuation mark.
    assert_eq!(
        super::breaks_for_test("1}2"),
        vec![(2, Allowed), (3, Mandatory)]
    );
    assert_eq!(
        super::breaks_for_test("1}2%"),
        vec![(2, Allowed), (4, Mandatory)]
    );
    assert_eq!(super::breaks_for_test("1}%"), vec![(3, Mandatory)]);
    assert_eq!(super::breaks_for_test("1,2"), vec![(3, Mandatory)]);
}

#[test]
fn unicode_16_lb8a_only_protects_the_immediate_zwj_boundary() {
    use super::BreakOpportunity::{Allowed, Mandatory};
    // UAX #14 revision 53: LB8a precedes LB9. In ID ZWJ CM ID, LB8a
    // protects ZWJ x CM, not CM / ID. LB9 ignores the attached ZWJ/CM
    // only for subsequent rules, leaving ID / ID allowed by LB31.
    assert_eq!(
        super::breaks_for_test("\u{4E00}\u{200D}\u{0308}\u{4E01}"),
        vec![(8, Allowed), (11, Mandatory)]
    );
    assert_eq!(
        super::breaks_for_test("\u{4E00}\u{0308}\u{200D}\u{4E01}"),
        vec![(11, Mandatory)]
    );
    assert_eq!(
        super::breaks_for_test("\u{4E00}\u{200D}\u{4E01}"),
        vec![(9, Mandatory)]
    );
}

#[test]
#[ignore = "requires LICHTUNG_UNICODE16_SOURCE_DIR with hash-verified original Unicode 16 sources"]
fn official_breaks() {
    for (name, grapheme, expected_count) in [
        ("auxiliary/GraphemeBreakTest.txt", true, 1093),
        ("auxiliary/LineBreakTest.txt", false, 16672),
    ] {
        let mut count = 0;
        let mut skipped = 0;
        let mut failures = Vec::new();
        for (line, row) in fixture(name).lines().enumerate() {
            let mut text = String::new();
            let mut expected = Vec::new();
            let mut representable = true;
            for token in row.split_whitespace() {
                match token {
                    "\u{00F7}" => expected.push(text.len()),
                    "\u{00D7}" => (),
                    cp => {
                        if let Some(scalar) = char::from_u32(u32::from_str_radix(cp, 16).unwrap()) {
                            text.push(scalar);
                        } else {
                            representable = false;
                            break;
                        }
                    }
                }
            }
            if !representable {
                skipped += 1;
                continue;
            }
            let actual: Vec<_> = if grapheme {
                text.grapheme_indices(true)
                    .map(|(offset, _)| offset)
                    .chain(std::iter::once(text.len()))
                    .collect()
            } else {
                super::breaks_for_test(&text)
                    .into_iter()
                    .map(|(offset, _)| offset)
                    .collect()
            };
            if actual != expected {
                failures.push((line + 1, row.to_owned(), actual, expected));
            }
            count += 1;
        }
        eprintln!("{name}: {count} representable, {skipped} non-scalar cases");
        assert_eq!(skipped, 0, "{name}: every official case must be exercised");
        assert!(
            failures.is_empty(),
            "{name}: {} failures: {:?}",
            failures.len(),
            &failures[..failures.len().min(20)]
        );
        assert_eq!(count + skipped, expected_count);
    }
}
