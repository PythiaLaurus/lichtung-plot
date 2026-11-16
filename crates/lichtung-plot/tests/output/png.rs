use super::*;

fn scatter() -> Plot<lichtung_plot::kind::Scatter> {
    let (x, y) = fixture_values();
    Plot::scatter(x, y).size(320, 240)
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
