use super::*;

fn scatter() -> Plot<lichtung_plot::kind::Scatter> {
    let (x, y) = fixture_values();
    Plot::scatter(x, y).size(320, 240)
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
fn png_options_reject_invalid_sizing_without_artifacts() {
    let directory = tempdir();
    for (index, sizing) in [
        PngSizing::Scale(0.0),
        PngSizing::Dpi(f32::NAN),
        PngSizing::Dpi(0.0),
        PngSizing::Scale(1_000_000.0),
    ]
    .into_iter()
    .enumerate()
    {
        let output = directory.join(format!("invalid-{index}.png"));
        let error = scatter()
            .png_with_options(
                &output,
                PngOptions {
                    sizing,
                    ..PngOptions::default()
                },
            )
            .err()
            .unwrap();
        assert!(matches!(error, lichtung_plot::Error::Render(_)), "{error}");
        assert!(!output.exists());
    }
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn plot_and_canvas_reject_unrepresentable_low_dpi_without_artifacts() {
    let directory = tempdir();
    let plot_output = directory.join("plot.png");
    let canvas_output = directory.join("canvas.png");
    let options = PngOptions {
        sizing: PngSizing::Dpi(0.01),
        compression: PngCompression::Balanced,
    };
    let plot = || {
        let (x, y) = fixture_values();
        Plot::scatter(x, y).size(10_000, 10_000)
    };

    let error = plot()
        .png_with_options(&plot_output, options)
        .err()
        .unwrap();
    assert!(error.to_string().contains("too small to encode"));
    let error = Canvas::grid(1, 1)
        .unwrap()
        .panel(0, 0, plot())
        .unwrap()
        .size(10_000, 10_000)
        .png_with_options(&canvas_output, options)
        .err()
        .unwrap();
    assert!(error.to_string().contains("too small to encode"));
    assert!(!plot_output.exists());
    assert!(!canvas_output.exists());
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn histogram_density_and_qq_share_svg_png_paths() {
    let value = NumericValues::new("value", [1.0, 2.0, 2.0, 3.0, 4.0]);
    let directory = unique_dir("distribution-png-paths");
    let histogram = Plot::histogram(lichtung_plot::HistogramInput::samples(value.clone()));
    let histogram = histogram
        .bins(HistogramBins::Count(4))
        .stat(HistogramStat::Density);
    for (name, plot) in [
        ("histogram", histogram.erase()),
        ("density", Plot::density(value.clone()).erase()),
        ("qq", Plot::qq(value).erase()),
    ] {
        let svg = directory.join(format!("{name}.svg"));
        let png = directory.join(format!("{name}.png"));
        let output = plot
            .svg_preserve_text(&svg)
            .unwrap()
            .png_preview(&png)
            .unwrap();
        assert!(std::fs::read_to_string(svg).unwrap().contains("<svg"));
        assert_eq!(
            png_dimensions(&std::fs::read(png).unwrap()),
            (output.width(), output.height())
        );
    }
    std::fs::remove_dir_all(directory).unwrap();
}
