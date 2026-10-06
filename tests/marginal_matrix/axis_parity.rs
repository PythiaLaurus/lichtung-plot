use lichtung_plot::{NumericValues, Plot, Theme};
use sha2::{Digest, Sha256};

#[cfg(all(unix, not(any(target_os = "macos", target_os = "android"))))]
fn is_frame_segment(node: roxmltree::Node<'_, '_>, stroke_width: &str, foreground: &str) -> bool {
    if !node.has_tag_name("polyline")
        || node.attribute("stroke-width") != Some(stroke_width)
        || node.attribute("stroke") != Some(foreground)
    {
        return false;
    }
    let points = node
        .attribute("points")
        .unwrap()
        .split_whitespace()
        .map(|point| {
            let (x, y) = point.split_once(',').unwrap();
            (x.parse::<f64>().unwrap(), y.parse::<f64>().unwrap())
        })
        .collect::<Vec<_>>();
    points.len() == 2
        && (points[1].0 - points[0].0).abs() + (points[1].1 - points[0].1).abs() > 50.0
}

#[cfg(all(unix, not(any(target_os = "macos", target_os = "android"))))]
fn assert_source_topology(bytes: &[u8], theme: Theme) {
    let (stroke_width, foreground) = match theme {
        Theme::Publication => ("0.7998046875", "#000000"),
        Theme::Nature => ("0.7998046875", "#282828"),
        Theme::Science => ("1", "#000000"),
        Theme::CHI => ("0.7998046875", "#3C3C3C"),
    };
    let document = roxmltree::Document::parse(std::str::from_utf8(bytes).unwrap()).unwrap();
    assert_eq!(
        document
            .descendants()
            .filter(|node| node.has_tag_name("circle"))
            .count(),
        8
    );
    assert_eq!(
        document
            .descendants()
            .filter(|node| node.has_tag_name("text"))
            .count(),
        24
    );
    let frames = document
        .descendants()
        .filter(|node| is_frame_segment(*node, stroke_width, foreground))
        .count();
    assert_eq!(
        frames, 8,
        "{theme:?}: default cell frames must remain bottom/left only"
    );
}

#[test]
fn default_matrix_full_svg_bytes_match_the_independent_committed_baseline() {
    // Captured at 633c8e0 using this exact public scene and font environment,
    // before the axis representation change. These cover the entire SVG.
    let cases = [
        (
            "publication",
            Theme::Publication,
            19_413,
            "400421d31a904b9bdefe4f4ed5d3dc0a9d12fa3e7893e2c70955e3a593c3e654",
        ),
        (
            "nature",
            Theme::Nature,
            19_332,
            "ddb322a2205c03adbed763785ea5c00bf6d3b49b3c9c798272839bd7b8abad5e",
        ),
        (
            "science",
            Theme::Science,
            19_028,
            "353bce392eaa79fc19aac3d1574d6e38450452e7a884eb20e4829dd50b59b632",
        ),
        (
            "chi",
            Theme::CHI,
            19_413,
            "79a8aa3e47ccd976c46c2ea48b5cfc2e3cfa34aa06ecd48dafa551c4d5e96d7a",
        ),
    ];
    let temporary = tempfile::tempdir().unwrap();
    let output = std::env::var_os("LICHTUNG_MATRIX_PARITY_OUTPUT")
        .map_or_else(|| temporary.path().to_path_buf(), std::path::PathBuf::from);
    std::fs::create_dir_all(&output).unwrap();
    for (name, theme, length, digest) in cases {
        let path = output.join(format!("{name}.svg"));
        Plot::matrix(vec![
            NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
            NumericValues::new("y", [4.0, 3.0, 2.0, 1.0]),
        ])
        .theme(theme)
        .size(800, 600)
        .svg_preserve_text(&path)
        .unwrap();
        let bytes = std::fs::read(&path).unwrap();
        if let Some(baseline) = std::env::var_os("LICHTUNG_MATRIX_BASELINE_DIR") {
            assert_eq!(
                bytes,
                std::fs::read(std::path::PathBuf::from(baseline).join(format!("{name}.svg")))
                    .unwrap(),
                "full {name} SVG differs from committed baseline"
            );
        }
        assert_eq!(bytes.len(), length, "{name}");
        assert_eq!(format!("{:x}", Sha256::digest(&bytes)), digest, "{name}");
        #[cfg(all(unix, not(any(target_os = "macos", target_os = "android"))))]
        assert_source_topology(&bytes, theme);
    }
}
