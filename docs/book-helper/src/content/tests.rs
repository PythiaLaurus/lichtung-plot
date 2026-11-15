use std::path::Path;

use super::{anchor, catalog, local_links, navigation, rendered_anchors, source, Outline};
use crate::{example_extension, examples_path, gallery, PYTHON_PACKAGE};

const CASE: &str = "first\tlichtung-plot\t01_quick_start\tquick_start.svg\tfirst-plot\tstruct.Plot.html\tshowcase\tFirst plot\t首张图\tOwned values\t自有数据";
const PYTHON_CASE: &str = "python-figures\tlichtung-plot-python\tscientific_composition\tscientific-composition.svg,scientific-composition-text.svg,scientific-composition-preview.png\tpython-figures\tpython\treference\tScientific views\t科学视图\tNative snapshots\t原生快照";

fn sample_png(width: u32, depth: u8) -> Vec<u8> {
    let mut header = Vec::new();
    header.extend(width.to_be_bytes());
    header.extend(1_u32.to_be_bytes());
    header.extend([depth, 6, 0, 0, 0]);
    // A stored zlib block for one filter byte and one transparent RGBA pixel.
    let data = [
        0x78, 0x01, 0x01, 0x05, 0x00, 0xfa, 0xff, 0, 0, 0, 0, 0, 0, 0x05, 0, 0x01,
    ];
    let mut bytes = b"\x89PNG\r\n\x1a\n".to_vec();
    for (kind, payload) in [
        (b"IHDR", header.as_slice()),
        (b"IDAT", data.as_slice()),
        (b"IEND", &[]),
    ] {
        bytes.extend(u32::try_from(payload.len()).unwrap().to_be_bytes());
        bytes.extend(kind);
        bytes.extend(payload);
        let mut crc = u32::MAX;
        for byte in kind.iter().chain(payload) {
            crc ^= u32::from(*byte);
            for _ in 0..8 {
                crc = (crc >> 1) ^ if crc & 1 == 0 { 0 } else { 0xedb8_8320 };
            }
        }
        bytes.extend((!crc).to_be_bytes());
    }
    bytes
}

#[test]
fn catalog_refuses_duplicate_execution_identities() {
    assert!(catalog(&format!("{CASE}\n{CASE}")).is_err());
    let other_package = CASE.replacen(
        "first\tlichtung-plot\t",
        "second\tlichtung-plot-ndarray\t",
        1,
    );
    assert_eq!(
        catalog(&format!("{CASE}\n{other_package}")).unwrap().len(),
        2
    );
    assert!(catalog(&CASE.replace("lichtung-plot\t", "unknown-package\t")).is_err());
    let text = format!("{CASE}\n{PYTHON_CASE}");
    let cases = catalog(&text).unwrap();
    assert_eq!(cases.len(), 2);
    assert_eq!(cases[1].package, PYTHON_PACKAGE);
    assert_eq!(
        cases[1].artifacts,
        [
            "scientific-composition.svg",
            "scientific-composition-text.svg",
            "scientific-composition-preview.png"
        ]
    );
    assert_eq!(
        examples_path(PYTHON_PACKAGE),
        Path::new("bindings/python/examples")
    );
    assert_eq!(example_extension(PYTHON_PACKAGE), "py");
    assert_eq!(example_extension("lichtung-plot"), "rs");
    assert!(
        catalog(&text.replace("\tpython\treference", "\tstruct.Plot.html\treference")).is_err()
    );
    assert!(catalog(&CASE.replace("struct.Plot.html", "https://example.com/api")).is_err());
    assert!(catalog(&format!(
        "{text}\n{}",
        PYTHON_CASE.replacen("python-figures\t", "other-python\t", 1)
    ))
    .is_err());
    let cards = gallery::cards(
        &catalog(&text.replace("\treference\t", "\tshowcase\t")).unwrap(),
        0,
    )
    .unwrap();
    assert!(cards.contains("href=\"#python\""));
    assert!(cards.contains("scientific_composition.py"));
    assert!(!cards.contains("docs.rs/lichtung-plot-python"));
}

#[test]
fn artifact_names_cannot_escape_the_fresh_case_directory() {
    assert!(catalog(&CASE.replace("quick_start.svg", "../previous.svg")).is_err());
    assert!(catalog(&CASE.replace("quick_start.svg", "previous.png")).is_err());
    assert!(catalog(&CASE.replace("quick_start.svg", "plot.svg,../previous.png")).is_err());
    assert!(catalog(&CASE.replace("quick_start.svg", "plot.svg,preview.jpg")).is_err());
    assert!(catalog(&CASE.replace("quick_start.svg", "plot.svg,plot.svg")).is_err());
    let valid = sample_png(1, 8);
    assert_eq!(&valid[29..33], &[0x1f, 0x15, 0xc4, 0x89]);
    assert!(gallery::png(&valid));
    assert!(!gallery::png(b"<svg></svg>"));
    assert!(!gallery::png(&valid[..33]));
    assert!(!gallery::png(&sample_png(0, 8)));
    assert!(!gallery::png(&sample_png(1, 3)));
    let mut invalid = valid;
    invalid[29] ^= 1;
    assert!(!gallery::png(&invalid));
}

#[test]
fn source_anchor_requires_one_complete_nonempty_pair() {
    assert!(anchor("// ANCHOR: case\nlet x = 1;\n// ANCHOR_END: case", "case").is_ok());
    assert!(anchor("// ANCHOR: case\nlet x = 1;", "case").is_err());
    assert!(anchor("# ANCHOR: case\nx = 1\n# ANCHOR_END: case", "case").is_ok());
    assert!(anchor("# ANCHOR: case\n \n# ANCHOR_END: case", "case").is_err());
    assert!(anchor("# ANCHOR: case\nx = 1\n// ANCHOR_END: case", "case").is_err());
    assert!(anchor(
        "# ANCHOR: case\n# ANCHOR: case\nx = 1\n# ANCHOR_END: case",
        "case"
    )
    .is_err());
    assert!(anchor(
        "# ANCHOR: case\nx = 1\n# ANCHOR_END: case\n# ANCHOR_END: case",
        "case"
    )
    .is_err());
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    assert_eq!(
        source(
            &root,
            &root.join("docs/en/src/sections"),
            "../../../../bindings/python/examples/scientific_composition.py:example"
        )
        .unwrap(),
        "lichtung-plot-python/scientific_composition.py:example"
    );
}

#[test]
fn language_navigation_cannot_silently_fall_back_to_the_homepage() {
    assert!(navigation("[中文](../zh-CN/index.html#axes)", 0, "axes").is_ok());
    assert!(navigation("[中文](../zh-CN/index.html#gallery)", 0, "axes").is_err());
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let cases = catalog(&format!("{CASE}\n{PYTHON_CASE}")).unwrap();
    let outline = Outline::default();
    assert!(local_links(
        &root,
        "[Source](examples/lichtung-plot-python/scientific_composition.py)",
        &outline,
        &cases
    )
    .is_ok());
    assert!(local_links(
        &root,
        "[Preview](gallery/python-figures/scientific-composition-preview.png)",
        &outline,
        &cases
    )
    .is_ok());
    assert!(local_links(
        &root,
        "[Wrong owner](examples/lichtung-plot/scientific_composition.py)",
        &outline,
        &cases
    )
    .is_err());
    assert!(local_links(
        &root,
        "[Missing](examples/lichtung-plot-python/missing.py)",
        &outline,
        &cases
    )
    .is_err());
    assert!(local_links(
        &root,
        "[Traversal](examples/lichtung-plot-python/../examples/scientific_composition.py)",
        &outline,
        &cases
    )
    .is_err());
}

#[test]
fn rendered_headings_cannot_shadow_explicit_walkthrough_anchors() {
    let page = "<a id=\"numeric-policies\"></a><h3 id=\"numeric-policies\">Numeric policies</h3>";
    assert!(rendered_anchors(page).is_err());
}
