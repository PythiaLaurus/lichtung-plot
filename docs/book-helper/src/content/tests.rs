use super::{anchor, catalog, navigation, rendered_anchors};

const CASE: &str = "first\t01_quick_start\tquick_start.svg\tfirst-plot\tstruct.Plot.html\tFirst plot\t首张图\tOwned values\t自有数据";

#[test]
fn catalog_refuses_duplicate_execution_identities() {
    assert!(catalog(&format!("{CASE}\n{CASE}")).is_err());
}

#[test]
fn artifact_names_cannot_escape_the_fresh_case_directory() {
    assert!(catalog(&CASE.replace("quick_start.svg", "../previous.svg")).is_err());
}

#[test]
fn source_anchor_requires_one_complete_nonempty_pair() {
    assert!(anchor("// ANCHOR: case\nlet x = 1;\n// ANCHOR_END: case", "case").is_ok());
    assert!(anchor("// ANCHOR: case\nlet x = 1;", "case").is_err());
}

#[test]
fn language_navigation_cannot_silently_fall_back_to_the_homepage() {
    assert!(navigation("[中文](../zh-CN/index.html#axes)", 0, "axes").is_ok());
    assert!(navigation("[中文](../zh-CN/index.html#gallery)", 0, "axes").is_err());
}

#[test]
fn rendered_headings_cannot_shadow_explicit_walkthrough_anchors() {
    let page = "<a id=\"numeric-policies\"></a><h3 id=\"numeric-policies\">Numeric policies</h3>";
    assert!(rendered_anchors(page).is_err());
}
