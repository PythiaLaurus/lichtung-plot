use super::*;

#[cfg(feature = "png")]
#[test]
fn scatter_png_smoke() {
    // Sanity: render a scatter to PNG via the SVG→`resvg`→PNG pipeline.
    let df = fixture_df();
    let dir = tempdir();
    let out = dir.join("scatter.png");
    let mut chart = Chart::new(&df);
    chart.x("x").y("y").size(320, 240).scatter();
    chart.render_to_png(&out).expect("png render must succeed");
    let bytes = std::fs::read(&out).expect("file must exist");
    // PNG magic: 89 50 4E 47
    assert!(bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]));
}

#[cfg(feature = "png")]
#[test]
fn png_options_scale_output_dimensions() {
    let df = fixture_df();
    let dir = tempdir();
    let out = dir.join("scatter-scale.png");
    let mut chart = Chart::new(&df);
    chart.x("x").y("y").size(320, 240).scatter();
    chart
        .render_to_png_with_options(
            &out,
            &PngOptions {
                sizing: PngSizing::Scale(2.0),
                ..PngOptions::default()
            },
        )
        .expect("png render must succeed");

    let bytes = std::fs::read(&out).expect("png must exist");
    assert_eq!(png_dimensions(&bytes), (640, 480));
    assert_eq!(png_phys(&bytes), None);
}

#[cfg(feature = "png")]
#[test]
fn png_options_dpi_scales_and_writes_metadata() {
    let df = fixture_df();
    let dir = tempdir();
    let out = dir.join("scatter-dpi.png");
    let mut chart = Chart::new(&df);
    chart.x("x").y("y").size(320, 240).scatter();
    chart
        .render_to_png_with_options(
            &out,
            &PngOptions {
                sizing: PngSizing::Dpi(192.0),
                ..PngOptions::default()
            },
        )
        .expect("png render must succeed");

    let bytes = std::fs::read(&out).expect("png must exist");
    assert_eq!(png_dimensions(&bytes), (640, 480));
    assert_eq!(png_phys(&bytes), Some((7559, 7559, 1)));
}

#[cfg(feature = "png")]
#[test]
fn png_options_width_and_height_preserve_aspect_ratio() {
    let df = fixture_df();
    let dir = tempdir();
    let width_out = dir.join("scatter-width.png");
    let height_out = dir.join("scatter-height.png");
    let mut chart = Chart::new(&df);
    chart.x("x").y("y").size(320, 240).scatter();
    chart
        .render_to_png_with_options(
            &width_out,
            &PngOptions {
                sizing: PngSizing::Width(800),
                ..PngOptions::default()
            },
        )
        .expect("width-sized png render must succeed");
    chart
        .render_to_png_with_options(
            &height_out,
            &PngOptions {
                sizing: PngSizing::Height(600),
                ..PngOptions::default()
            },
        )
        .expect("height-sized png render must succeed");

    assert_eq!(
        png_dimensions(&std::fs::read(&width_out).expect("png must exist")),
        (800, 600)
    );
    assert_eq!(
        png_dimensions(&std::fs::read(&height_out).expect("png must exist")),
        (800, 600)
    );
}

#[cfg(feature = "png")]
#[test]
fn png_options_reject_invalid_scale() {
    let df = fixture_df();
    let dir = tempdir();
    let out = dir.join("scatter-invalid.png");
    let mut chart = Chart::new(&df);
    chart.x("x").y("y").size(320, 240).scatter();
    let err = chart
        .render_to_png_with_options(
            &out,
            &PngOptions {
                sizing: PngSizing::Scale(0.0),
                ..PngOptions::default()
            },
        )
        .unwrap_err();
    assert!(err.to_string().contains("png scale must be positive"));
}

#[cfg(feature = "png")]
#[test]
fn scatter_svg_and_png_reuses_persistent_svg() {
    let df = fixture_df();
    let dir = tempdir();
    let svg = dir.join("scatter.svg");
    let png = dir.join("scatter.png");
    let tmp_svg = dir.join("scatter.png.lichtung-tmp.svg");
    let mut chart = Chart::new(&df);
    chart.x("x").y("y").size(320, 240).scatter();

    chart
        .render_to_svg_and_png(&svg, &png)
        .expect("combined render must succeed");

    assert!(std::fs::read_to_string(&svg)
        .expect("svg must exist")
        .contains("<svg"));
    let png_bytes = std::fs::read(&png).expect("png must exist");
    assert!(png_bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]));
    assert!(!tmp_svg.exists(), "combined render must not leave temp SVG");
}

#[cfg(feature = "png")]
#[test]
fn histogram_svg_and_png_share_the_canonical_scene() {
    let df = df!("value" => [1.0_f64, 2.0, 2.0, 3.0, 4.0]).unwrap();
    let dir = unique_dir("histogram-png-parity");
    let svg = dir.join("histogram.svg");
    let png = dir.join("histogram.png");
    let mut chart = Chart::new(&df);
    let _ = chart.x("value");
    chart
        .histogram_bins(HistogramBins::Count(4))
        .histogram_stat(HistogramStat::Density)
        .histogram();

    chart.render_to_svg_and_png(&svg, &png).unwrap();

    assert!(std::fs::read_to_string(svg)
        .unwrap()
        .contains(">Density</text>"));
    let png = std::fs::read(png).unwrap();
    assert_eq!(png_dimensions(&png), (640, 480));
    std::fs::remove_dir_all(dir).unwrap();
}

#[cfg(feature = "png")]
#[test]
fn density_and_qq_render_through_png_path() {
    let df = df!("value" => [1.0_f64, 2.0, 3.0, 4.0, 5.0]).unwrap();
    let dir = unique_dir("density-qq-png");
    for kind in ["density", "qq"] {
        let svg = dir.join(format!("{kind}.svg"));
        let png = dir.join(format!("{kind}.png"));
        let mut chart = Chart::new(&df);
        let _ = chart.x("value");
        if kind == "density" {
            chart.density();
        } else {
            chart.qq();
        }
        chart.render_to_svg_and_png(&svg, &png).unwrap();
        assert_eq!(png_dimensions(&std::fs::read(png).unwrap()), (640, 480));
    }
    std::fs::remove_dir_all(dir).unwrap();
}
