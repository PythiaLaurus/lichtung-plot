use super::*;

fn scatter() -> Plot<lichtung_plot::kind::Scatter> {
    let frame = fixture_df();
    Plot::scatter(data::numeric(&frame, "x"), data::numeric(&frame, "y")).size(320, 240)
}

#[test]
fn scatter_png_smoke() {
    let directory = tempdir();
    let output = directory.join("scatter.png");
    scatter().png(&output).unwrap();
    let bytes = std::fs::read(output).unwrap();
    assert_eq!(png_dimensions(&bytes), (1000, 750));
    assert_eq!(png_phys(&bytes), Some((11_811, 11_811, 1)));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn png_options_default_to_300_dpi_and_balanced_compression() {
    assert_eq!(
        PngOptions::default(),
        PngOptions {
            sizing: PngSizing::Dpi(300.0),
            compression: PngCompression::Balanced,
        }
    );
    assert_eq!(PngCompression::default(), PngCompression::Balanced);
}

#[test]
fn png_with_options_uses_exact_scale_and_dpi() {
    let directory = tempdir();
    let scale = directory.join("scale.png");
    let dpi = directory.join("dpi.png");
    scatter()
        .png_with_options(
            &scale,
            PngOptions {
                sizing: PngSizing::Scale(2.0),
                compression: PngCompression::Fast,
            },
        )
        .unwrap();
    scatter()
        .png_with_options(
            &dpi,
            PngOptions {
                sizing: PngSizing::Dpi(192.0),
                compression: PngCompression::High,
            },
        )
        .unwrap();
    let scale = std::fs::read(scale).unwrap();
    let dpi = std::fs::read(dpi).unwrap();
    assert_eq!(png_dimensions(&scale), (640, 480));
    assert_eq!(png_phys(&scale), None);
    assert_eq!(png_dimensions(&dpi), (640, 480));
    assert_eq!(png_phys(&dpi), Some((7559, 7559, 1)));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn png_width_and_height_preserve_aspect_ratio() {
    let directory = tempdir();
    for (name, sizing) in [
        ("width.png", PngSizing::Width(800)),
        ("height.png", PngSizing::Height(600)),
    ] {
        let output = directory.join(name);
        scatter()
            .png_with_options(
                &output,
                PngOptions {
                    sizing,
                    ..PngOptions::default()
                },
            )
            .unwrap();
        assert_eq!(png_dimensions(&std::fs::read(output).unwrap()), (800, 600));
    }
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn png_with_options_rejects_invalid_scale_without_artifact() {
    let directory = tempdir();
    let output = directory.join("invalid.png");
    let error = scatter()
        .png_with_options(
            &output,
            PngOptions {
                sizing: PngSizing::Scale(0.0),
                ..PngOptions::default()
            },
        )
        .err()
        .unwrap();
    assert!(error.to_string().contains("png scale must be positive"));
    assert!(!output.exists());
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn histogram_density_and_qq_share_svg_png_paths() {
    let frame = df!("value" => [1.0_f64, 2.0, 2.0, 3.0, 4.0]).unwrap();
    let directory = unique_dir("distribution-png-paths");
    let histogram = Plot::histogram(lichtung_plot::HistogramInput::samples(data::numeric(
        &frame, "value",
    )));
    let histogram = histogram
        .bins(HistogramBins::Count(4))
        .stat(HistogramStat::Density);
    for (name, plot) in [
        ("histogram", histogram.erase()),
        (
            "density",
            Plot::density(data::numeric(&frame, "value")).erase(),
        ),
        ("qq", Plot::qq(data::numeric(&frame, "value")).erase()),
    ] {
        let svg = directory.join(format!("{name}.svg"));
        let png = directory.join(format!("{name}.png"));
        plot.svg(&svg).unwrap().png_preview(&png).unwrap();
        assert!(std::fs::read_to_string(svg).unwrap().contains("<svg"));
        assert_eq!(png_dimensions(&std::fs::read(png).unwrap()), (640, 480));
    }
    std::fs::remove_dir_all(directory).unwrap();
}
