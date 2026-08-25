use super::{
    assert_no_staging_files, publish_staged, reserve_unique_sibling_temp_path, test_directory,
    with_temporary_artifacts, Error,
};
#[cfg(feature = "png")]
use super::{configured_chart, fixture, PlotKind};

#[test]
fn render_failure_preserves_existing_svg_and_removes_staging_files() {
    let dir = test_directory("failed-svg-publication");
    let output = dir.join("canvas.svg");
    std::fs::write(&output, "existing artifact").unwrap();
    let chart = crate::Plot::scatter(
        crate::NumericValues::new("x", [1.0, 2.0]),
        crate::NumericValues::new("y", [1.0]),
    );

    assert!(chart.svg(&output).is_err());
    assert_eq!(
        std::fs::read_to_string(&output).unwrap(),
        "existing artifact"
    );
    assert_no_staging_files(&dir);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn finalization_failure_removes_staging_files_without_touching_destination() {
    let dir = test_directory("failed-finalization-cleanup");
    let output = dir.join("canvas.svg");
    std::fs::write(&output, "existing artifact").unwrap();
    let staged = reserve_unique_sibling_temp_path(&output, "svg").unwrap();

    let result: Result<(), Error> = with_temporary_artifacts(vec![staged.clone()], || {
        std::fs::write(&staged, "partial staged artifact")?;
        Err(Error::Render("backend end failed".into()))
    });

    assert!(result.is_err());
    assert_eq!(
        std::fs::read_to_string(&output).unwrap(),
        "existing artifact"
    );
    assert_no_staging_files(&dir);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn publication_rename_failure_removes_staging_files_and_preserves_destination() {
    let dir = test_directory("failed-rename-cleanup");
    let output = dir.join("canvas.svg");
    std::fs::create_dir(&output).unwrap();
    std::fs::write(output.join("existing"), "existing artifact").unwrap();
    let staged = reserve_unique_sibling_temp_path(&output, "svg").unwrap();

    let result = with_temporary_artifacts(vec![staged.clone()], || {
        std::fs::write(&staged, "complete staged artifact")?;
        publish_staged(&staged, &output)
    });

    assert!(result.is_err());
    assert_eq!(
        std::fs::read_to_string(output.join("existing")).unwrap(),
        "existing artifact"
    );
    assert_no_staging_files(&dir);
    std::fs::remove_dir_all(dir).unwrap();
}

#[cfg(feature = "png")]
#[test]
fn rasterization_failure_preserves_existing_png_and_removes_staging_files() {
    use crate::spec::{PngCompression, PngOptions, PngSizing};

    let dir = test_directory("failed-png-publication");
    let output = dir.join("canvas.png");
    std::fs::write(&output, b"existing png artifact").unwrap();
    let data = fixture();
    let options = PngOptions {
        sizing: PngSizing::Scale(0.0),
        compression: PngCompression::Balanced,
    };

    assert!(configured_chart(&data, PlotKind::Scatter)
        .png_with_options(&output, options)
        .is_err());
    assert_eq!(std::fs::read(&output).unwrap(), b"existing png artifact");
    assert_no_staging_files(&dir);
    std::fs::remove_dir_all(dir).unwrap();
}

#[cfg(feature = "png")]
#[test]
fn chained_outputs_publish_each_file_independently() {
    let dir = test_directory("successful-chained-publication");
    let svg = dir.join("canvas.svg");
    let png = dir.join("canvas.png");
    let data = fixture();

    configured_chart(&data, PlotKind::Scatter)
        .svg(&svg)
        .unwrap()
        .png_preview(&png)
        .unwrap();

    assert!(std::fs::read_to_string(&svg).unwrap().starts_with("<svg"));
    assert!(std::fs::read(&png)
        .unwrap()
        .starts_with(&[0x89, b'P', b'N', b'G']));
    assert_no_staging_files(&dir);
    std::fs::remove_dir_all(dir).unwrap();
}
